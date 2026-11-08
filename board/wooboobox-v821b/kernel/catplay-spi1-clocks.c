// SPDX-License-Identifier: GPL-2.0-only
/* SPI1 clocks missing from the CatPlay v0.22 V821 CCU implementation.
 * Register definitions: Allwinner BSP ccu-sun300iw1-app.c, SPI1.
 * Scope: single-core V821B board with no other user of these SPI1 bits.
 */
#include <linux/clk-provider.h>
#include <linux/io.h>
#include <linux/module.h>
#include <linux/of.h>
#include <linux/platform_device.h>
#include <linux/reset-controller.h>
#include <linux/spinlock.h>

#define MOD 0x64
#define BUS 0x84
#define RST 0x94
#define SPI1_BIT BIT(19)
#define DIV_MASK (GENMASK(3, 0) | GENMASK(17, 16))

struct spi1_ccu {
    void __iomem *base;
    spinlock_t lock;
    struct clk_hw mod;
    struct reset_controller_dev reset;
};
#define to_ccu(hw) container_of(hw, struct spi1_ccu, mod)

static unsigned long choose_rate(unsigned long wanted, unsigned long parent,
                                 unsigned int *best_m, unsigned int *best_p)
{
    unsigned long best = 0, error = ~0UL;
    unsigned int m, p;
    for (p = 0; p < 4; p++) for (m = 1; m <= 16; m++) {
        unsigned long rate = parent / (m << p);
        unsigned long delta = rate > wanted ? rate - wanted : wanted - rate;
        if (delta < error) { best = rate; error = delta; *best_m = m; *best_p = p; }
    }
    return best;
}

static unsigned long mod_recalc(struct clk_hw *hw, unsigned long parent)
{
    u32 v = readl(to_ccu(hw)->base + MOD);
    return parent / (((v & 15) + 1) << ((v >> 16) & 3));
}

static int mod_determine(struct clk_hw *hw, struct clk_rate_request *request)
{
    unsigned int m = 1, p = 0;
    request->best_parent_hw = clk_hw_get_parent(hw);
    request->best_parent_rate = clk_hw_get_rate(request->best_parent_hw);
    request->rate = choose_rate(request->rate, request->best_parent_rate, &m, &p);
    if (request->rate < request->min_rate || request->rate > request->max_rate) return -ERANGE;
    return 0;
}

static int mod_set(struct clk_hw *hw, unsigned long rate, unsigned long parent)
{
    struct spi1_ccu *ccu = to_ccu(hw);
    unsigned int m = 1, p = 0;
    unsigned long flags;
    u32 value;
    choose_rate(rate, parent, &m, &p);
    spin_lock_irqsave(&ccu->lock, flags);
    value = readl(ccu->base + MOD);
    value = (value & ~DIV_MASK) | (m - 1) | (p << 16);
    writel(value, ccu->base + MOD);
    spin_unlock_irqrestore(&ccu->lock, flags);
    return 0;
}

static int mod_enable(struct clk_hw *hw)
{
    struct spi1_ccu *ccu = to_ccu(hw);
    unsigned long flags;
    spin_lock_irqsave(&ccu->lock, flags);
    writel(readl(ccu->base + MOD) | BIT(31), ccu->base + MOD);
    spin_unlock_irqrestore(&ccu->lock, flags);
    return 0;
}

static void mod_disable(struct clk_hw *hw)
{
    struct spi1_ccu *ccu = to_ccu(hw);
    unsigned long flags;
    spin_lock_irqsave(&ccu->lock, flags);
    writel(readl(ccu->base + MOD) & ~BIT(31), ccu->base + MOD);
    spin_unlock_irqrestore(&ccu->lock, flags);
}

static int mod_is_enabled(struct clk_hw *hw) { return !!(readl(to_ccu(hw)->base + MOD) & BIT(31)); }
static const struct clk_ops mod_ops = {
    .recalc_rate = mod_recalc, .determine_rate = mod_determine, .set_rate = mod_set,
    .enable = mod_enable, .disable = mod_disable, .is_enabled = mod_is_enabled,
};

static int reset_update(struct reset_controller_dev *reset, bool asserted)
{
    struct spi1_ccu *ccu = container_of(reset, struct spi1_ccu, reset);
    unsigned long flags;
    u32 value;
    spin_lock_irqsave(&ccu->lock, flags);
    value = readl(ccu->base + RST);
    writel(asserted ? value & ~SPI1_BIT : value | SPI1_BIT, ccu->base + RST);
    spin_unlock_irqrestore(&ccu->lock, flags);
    return 0;
}
static int reset_assert(struct reset_controller_dev *r, unsigned long id) { return reset_update(r, true); }
static int reset_deassert(struct reset_controller_dev *r, unsigned long id) { return reset_update(r, false); }
static const struct reset_control_ops reset_ops = { .assert = reset_assert, .deassert = reset_deassert };

static int spi1_probe(struct platform_device *pdev)
{
    struct device *dev = &pdev->dev;
    struct spi1_ccu *ccu;
    struct clk_hw_onecell_data *clocks;
    struct resource *res;
    static const char *const parents[] = { "dcxo" };
    const struct clk_init_data init = { .name = "catplay-spi1-mod", .ops = &mod_ops,
                                       .parent_names = parents, .num_parents = 1 };
    int ret;
    ccu = devm_kzalloc(dev, sizeof(*ccu), GFP_KERNEL);
    clocks = devm_kzalloc(dev, struct_size(clocks, hws, 2), GFP_KERNEL);
    if (!ccu || !clocks) return -ENOMEM;
    res = platform_get_resource(pdev, IORESOURCE_MEM, 0);
    if (!res || res->start != 0x42001000 || resource_size(res) < 0x98) return -EINVAL;
    /* Parent CCU already reserves this region. Touch only unused SPI1 fields.
     * irqsave protects shared gate words on this single-core board. Never load
     * alongside a CCU implementation which already implements SPI1.
     */
    ccu->base = devm_ioremap(dev, res->start, resource_size(res));
    if (!ccu->base) return -ENOMEM;
    spin_lock_init(&ccu->lock);
    // Select the known 40 MHz dcxo parent, keep SPI1 gated until requested.
    writel(0, ccu->base + MOD);
    ccu->mod.init = &init;
    ret = devm_clk_hw_register(dev, &ccu->mod);
    if (ret) return ret;
    clocks->num = 2;
    clocks->hws[0] = devm_clk_hw_register_gate(dev, "catplay-spi1-bus", "dcxo", 0,
                                             ccu->base + BUS, 19, 0, &ccu->lock);
    if (IS_ERR(clocks->hws[0])) return PTR_ERR(clocks->hws[0]);
    clocks->hws[1] = &ccu->mod;
    ccu->reset.owner = THIS_MODULE;
    ccu->reset.of_node = dev->of_node;
    ccu->reset.nr_resets = 1;
    ccu->reset.ops = &reset_ops;
    ret = devm_reset_controller_register(dev, &ccu->reset);
    if (ret) return ret;
    ret = devm_of_clk_add_hw_provider(dev, of_clk_hw_onecell_get, clocks);
    if (!ret) dev_info(dev, "SPI1 clock/reset extension registered\n");
    return ret;
}
static const struct of_device_id spi1_match[] = { { .compatible = "catplay,v821-spi1-clocks" }, {} };
MODULE_DEVICE_TABLE(of, spi1_match);
static struct platform_driver spi1_driver = { .probe = spi1_probe,
    .driver = { .name = "catplay-spi1-clocks", .of_match_table = spi1_match } };
module_platform_driver(spi1_driver);
MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("SPI1 clock/reset extension for CatPlay v0.22 V821B");
