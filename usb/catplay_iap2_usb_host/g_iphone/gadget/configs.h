// SPDX-License-Identifier: GPL-2.0
#pragma once
#include "libcomposite/composite.h"

/*
    {.id = 5, .s = "PTP"},
    {.id = 6, .s = "iPod USB Interface"},
    {.id = 7, .s = "PTP + Apple Mobile Device"},
    {.id = 8, .s = "PTP + Apple Mobile Device + Apple USB Ethernet"},
*/

static const struct usb_device_descriptor iphone_device_desc = {
	.bLength = USB_DT_DEVICE_SIZE,
	.bDescriptorType = USB_DT_DEVICE,
	.bcdUSB = cpu_to_le16(0x0210),
	.bDeviceClass = 0,
	.bDeviceSubClass = 0,
	.bDeviceProtocol = 0,
	.bMaxPacketSize0 = 64,
	.idVendor = cpu_to_le16(0x05ac),
	.idProduct = cpu_to_le16(0x12a8),
	.bcdDevice = cpu_to_le16(0x1602),
	.iManufacturer = 1,
	.iProduct = 2,
	.iSerialNumber = 3,
	.bNumConfigurations = 4,
};

static const struct usb_configuration iphone_configs[] = {
    {
        .label = "PTP",
        .bConfigurationValue = 1,
        .iConfiguration = 5,
        .bmAttributes = USB_CONFIG_ATT_ONE | USB_CONFIG_ATT_SELFPOWER,
        .MaxPower = 500,
    },
    {
        .label = "iPod USB Interface",
        .bConfigurationValue = 2,
        .iConfiguration = 6,
        .bmAttributes = USB_CONFIG_ATT_ONE | USB_CONFIG_ATT_SELFPOWER,
        .MaxPower = 500,
    },
    {
        .label = "PTP + Apple Mobile Device",
        .bConfigurationValue = 3,
        .iConfiguration = 7,
        .bmAttributes = USB_CONFIG_ATT_ONE | USB_CONFIG_ATT_SELFPOWER,
        .MaxPower = 500,
    },
    {
        .label = "PTP + Apple Mobile Device + Apple USB Ethernet",
        .bConfigurationValue = 4,
        .iConfiguration = 8,
        .bmAttributes = USB_CONFIG_ATT_ONE | USB_CONFIG_ATT_SELFPOWER,
        .MaxPower = 500,
    },
};
