// SPDX-License-Identifier: GPL-2.0
#pragma once

#include <linux/kernel.h>
#include <linux/module.h>
#include "libcomposite/composite.h"
#include "strings.h"
#include "configs.h"
#include "g_iphone.h"

struct f_iphone /* container_of: usb_function */
{
	struct usb_function func;
	struct g_iphone* g;
	int config;
};

struct f_iphone_usb_config { /* container_of: usb_configuration */
	struct usb_configuration cfg;
	struct g_iphone *g;
};

int iphone_do_config(struct usb_configuration *c);
