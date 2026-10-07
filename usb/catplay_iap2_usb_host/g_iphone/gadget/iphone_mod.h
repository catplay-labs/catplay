// SPDX-License-Identifier: GPL-2.0
#pragma once

/* Module params */
static char *iphone_serial = DEFAULT_IPHONE_SERIAL;
module_param(iphone_serial, charp, 0);
MODULE_PARM_DESC(iphone_serial, "iPhone serial number override");

static char *udc_name;
module_param(udc_name, charp, 0);
MODULE_PARM_DESC(udc_name, "Name of UDC to bind to");

static char *device_name;
module_param(device_name, charp, 0);
MODULE_PARM_DESC(device_name, "Name of default iPhone device to create on module load");
