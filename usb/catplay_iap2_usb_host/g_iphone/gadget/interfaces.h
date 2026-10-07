// SPDX-License-Identifier: GPL-2.0
#pragma once

#include "libcomposite/composite.h"
#include <uapi/linux/usb/audio.h>
#include "interfaces_audio.h"

/* HID class descriptor (per USB HID spec) */
struct usb_hid_descriptor
{
    __u8 bLength;
    __u8 bDescriptorType; /* 0x21 */
    __le16 bcdHID;
    __u8 bCountryCode;
    __u8 bNumDescriptors;

    struct
    {
        __u8 bDescriptorType; /* 0x22 = Report */
        __le16 wDescriptorLength;
    } __packed desc[1];
} __packed;

#ifndef USB_DT_HID
#define USB_DT_HID 0x21
#endif

#ifndef USB_DT_REPORT
#define USB_DT_REPORT 0x22
#endif

/*static struct usb_device_descriptor dev_desc1 = {
    .bLength = 18,
    .bDescriptorType = 1,
    .bcdUSB = 0x0210,
    .bDeviceClass = 0,
    .bDeviceSubClass = 0,
    .bDeviceProtocol = 0,
    .bMaxPacketSize0 = 64,
    .idVendor = 0x05ac,
    .idProduct = 0x12a8,
    .bcdDevice = 0x1602,
    .iManufacturer = 1,
    .iProduct = 2,
    .iSerialNumber = 3,
    .bNumConfigurations = 4,
};*/

/*static struct usb_config_descriptor cfg1 = {
    .bLength = 9,
    .bDescriptorType = 2,
    .wTotalLength = 0x0027,
    .bNumInterfaces = 1,
    .bConfigurationValue = 1,
    .iConfiguration = 5,
    .bmAttributes = 0xc0,
    .bMaxPower = 250, // 500 mA
};*/

static struct usb_interface_descriptor intf1 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 0,
    .bAlternateSetting = 0,
    .bNumEndpoints = 3,
    .bInterfaceClass = 6,
    .bInterfaceSubClass = 1,
    .bInterfaceProtocol = 1,
    .iInterface = 22,
};

static struct usb_endpoint_descriptor ep1 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x02,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

static struct usb_endpoint_descriptor ep2 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x81,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

static struct usb_endpoint_descriptor ep3 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x83,
    .bmAttributes = 0x03,
    .wMaxPacketSize = 0x0040,
    .bInterval = 10,
};

/*static struct usb_config_descriptor cfg2 = {
    .bLength = 9,
    .bDescriptorType = 2,
    .wTotalLength = 0x0095,
    .bNumInterfaces = 3,
    .bConfigurationValue = 2,
    .iConfiguration = 6,
    .bmAttributes = 0xc0,
    .bMaxPower = 250, // 500 mA
};*/

static struct usb_interface_descriptor intf2 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 0,
    .bAlternateSetting = 0,
    .bNumEndpoints = 0,
    .bInterfaceClass = 1,
    .bInterfaceSubClass = 1,
    .bInterfaceProtocol = 0,
    .iInterface = 0,
};


static struct usb_interface_descriptor intf3 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 1,
    .bAlternateSetting = 0,
    .bNumEndpoints = 0,
    .bInterfaceClass = 1,
    .bInterfaceSubClass = 2,
    .bInterfaceProtocol = 0,
    .iInterface = 0,
};

static struct usb_interface_descriptor intf4 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 1,
    .bAlternateSetting = 1,
    .bNumEndpoints = 1,
    .bInterfaceClass = 1,
    .bInterfaceSubClass = 2,
    .bInterfaceProtocol = 0,
    .iInterface = 0,
};

static struct usb_endpoint_descriptor ep4 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x81,
    .bmAttributes = 0x01,
    .wMaxPacketSize = 0x00c0,
    .bInterval = 4,
    .bRefresh = 0,
    .bSynchAddress = 0};

static struct
{
    struct usb_descriptor_header header;
    u8 raw[5];
} __packed cs_ep1 = {
    .header = {
        .bLength = 7,
        .bDescriptorType = USB_DT_CS_ENDPOINT,
    },
    .raw = {
        0x01,
        0x01,
        0x00,
        0x00,
        0x00,
    },
};

static struct usb_interface_descriptor intf5 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 2,
    .bAlternateSetting = 0,
    .bNumEndpoints = 1,
    .bInterfaceClass = 3,
    .bInterfaceSubClass = 0,
    .bInterfaceProtocol = 0,
    .iInterface = 0,
};

static struct usb_hid_descriptor hid_desc1 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_HID, /* 0x21 */
    .bcdHID = cpu_to_le16(0x0111),
    .bCountryCode = 0x00,
    .bNumDescriptors = 1,
    .desc = {{
        .bDescriptorType = USB_DT_REPORT, /* 0x22 */
        .wDescriptorLength = cpu_to_le16(208),
    }},
};
/* desc[0] type=0x22 len=208 */

static struct usb_endpoint_descriptor ep5 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x83,
    .bmAttributes = 0x03,
    .wMaxPacketSize = 0x0040,
    .bInterval = 1,
};

/*static struct usb_config_descriptor cfg3 = {
    .bLength = 9,
    .bDescriptorType = 2,
    .wTotalLength = 0x003e,
    .bNumInterfaces = 2,
    .bConfigurationValue = 3,
    .iConfiguration = 7,
    .bmAttributes = 0xc0,
    .bMaxPower = 250, // 500 mA
};*/

static struct usb_interface_descriptor intf6 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 0,
    .bAlternateSetting = 0,
    .bNumEndpoints = 3,
    .bInterfaceClass = 6,
    .bInterfaceSubClass = 1,
    .bInterfaceProtocol = 1,
    .iInterface = 22,
};

static struct usb_endpoint_descriptor ep6 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x02,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

static struct usb_endpoint_descriptor ep7 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x81,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

static struct usb_endpoint_descriptor ep8 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x83,
    .bmAttributes = 0x03,
    .wMaxPacketSize = 0x0040,
    .bInterval = 10,
};

static struct usb_interface_descriptor intf7 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 1,
    .bAlternateSetting = 0,
    .bNumEndpoints = 2,
    .bInterfaceClass = 255,
    .bInterfaceSubClass = 254,
    .bInterfaceProtocol = 2,
    .iInterface = 16,
};

static struct usb_endpoint_descriptor ep9 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x04,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

static struct usb_endpoint_descriptor ep10 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x85,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

/*static struct usb_config_descriptor cfg4 = {
    .bLength = 9,
    .bDescriptorType = 2,
    .wTotalLength = 0x0075,
    .bNumInterfaces = 3,
    .bConfigurationValue = 4,
    .iConfiguration = 8,
    .bmAttributes = 0xc0,
    .bMaxPower = 250, // 500 mA
};*/

static struct usb_interface_descriptor intf8 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 0,
    .bAlternateSetting = 0,
    .bNumEndpoints = 3,
    .bInterfaceClass = 6,
    .bInterfaceSubClass = 1,
    .bInterfaceProtocol = 1,
    .iInterface = 22,
};

static struct usb_endpoint_descriptor ep11 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x02,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

static struct usb_endpoint_descriptor ep12 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x81,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

static struct usb_endpoint_descriptor ep13 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x83,
    .bmAttributes = 0x03,
    .wMaxPacketSize = 0x0040,
    .bInterval = 10,
};

static struct usb_interface_descriptor intf9 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 1,
    .bAlternateSetting = 0,
    .bNumEndpoints = 2,
    .bInterfaceClass = 255,
    .bInterfaceSubClass = 254,
    .bInterfaceProtocol = 2,
    .iInterface = 16,
};

static struct usb_endpoint_descriptor ep14 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x04,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

static struct usb_endpoint_descriptor ep15 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x85,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

static struct usb_interface_descriptor intf10 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 2,
    .bAlternateSetting = 0,
    .bNumEndpoints = 0,
    .bInterfaceClass = 255,
    .bInterfaceSubClass = 253,
    .bInterfaceProtocol = 1,
    .iInterface = 20,
};

static struct usb_interface_descriptor intf11 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 2,
    .bAlternateSetting = 1,
    .bNumEndpoints = 2,
    .bInterfaceClass = 255,
    .bInterfaceSubClass = 253,
    .bInterfaceProtocol = 1,
    .iInterface = 20,
};

static struct usb_endpoint_descriptor ep16 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x86,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

static struct usb_endpoint_descriptor ep17 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x05,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

static struct usb_interface_descriptor intf12 = {
    .bLength = 9,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 2,
    .bAlternateSetting = 2,
    .bNumEndpoints = 2,
    .bInterfaceClass = 255,
    .bInterfaceSubClass = 253,
    .bInterfaceProtocol = 1,
    .iInterface = 20,
};

static struct usb_endpoint_descriptor ep18 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x86,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

static struct usb_endpoint_descriptor ep19 = {
    .bLength = 7,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x05,
    .bmAttributes = 0x02,
    .wMaxPacketSize = 0x0200,
    .bInterval = 0,
};

// 1 intf
static struct usb_descriptor_header *iphone_fs_descs1[] = {
    (struct usb_descriptor_header *)&intf1,
    (struct usb_descriptor_header *)&ep1,
    (struct usb_descriptor_header *)&ep2,
    (struct usb_descriptor_header *)&ep3,
    NULL,
};

// 4 intf
static struct usb_descriptor_header *iphone_fs_descs2[] = {
    (struct usb_descriptor_header *)&intf2,
    (struct usb_descriptor_header *)&cs_intf1,
    (struct usb_descriptor_header *)&cs_intf2,
    (struct usb_descriptor_header *)&cs_intf3,

    (struct usb_descriptor_header *)&intf3,
    (struct usb_descriptor_header *)&intf4,
    (struct usb_descriptor_header *)&cs_intf4,
    (struct usb_descriptor_header *)&cs_intf5,
    (struct usb_descriptor_header *)&ep4,
    (struct usb_descriptor_header *)&cs_ep1,

    (struct usb_descriptor_header *)&intf5,
    (struct usb_descriptor_header *)&hid_desc1,
    (struct usb_descriptor_header *)&ep5,
    NULL,
};

// 2 intf
static struct usb_descriptor_header *iphone_fs_descs3[] = {
    (struct usb_descriptor_header *)&intf6,
    (struct usb_descriptor_header *)&ep6,
    (struct usb_descriptor_header *)&ep7,
    (struct usb_descriptor_header *)&ep8,
    (struct usb_descriptor_header *)&intf7,
    (struct usb_descriptor_header *)&ep9,
    (struct usb_descriptor_header *)&ep10,
    NULL,
};

// 5 intf
static struct usb_descriptor_header *iphone_fs_descs4[] = {
    (struct usb_descriptor_header *)&intf8,
    (struct usb_descriptor_header *)&ep11,
    (struct usb_descriptor_header *)&ep12,
    (struct usb_descriptor_header *)&ep13,
    (struct usb_descriptor_header *)&intf9,
    (struct usb_descriptor_header *)&ep14,
    (struct usb_descriptor_header *)&ep15,
    (struct usb_descriptor_header *)&intf10,
    (struct usb_descriptor_header *)&intf11,
    (struct usb_descriptor_header *)&ep16,
    (struct usb_descriptor_header *)&ep17,
    (struct usb_descriptor_header *)&intf12,
    (struct usb_descriptor_header *)&ep18,
    (struct usb_descriptor_header *)&ep19,
    NULL,
};

static struct usb_descriptor_header **iphone_descs[] = {
    NULL,             /* index 0 unused */
    iphone_fs_descs1, /* bConfigurationValue = 1 */
    iphone_fs_descs2, /* bConfigurationValue = 2 */
    iphone_fs_descs3, /* bConfigurationValue = 3 */
    iphone_fs_descs4, /* bConfigurationValue = 4 */
};

static const int iphone_intf_counts[5] = {0, 1, 3, 2, 3};
