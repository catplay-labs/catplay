#pragma once

#include "libcomposite/composite.h"

#define DEFAULT_IPHONE_SERIAL "00008130000E044E384B1D3ADDDDDDD000BADCAD"

static char iphone_serial_buf[41] = DEFAULT_IPHONE_SERIAL;
static char iphone_serial_r_buf[64] = DEFAULT_IPHONE_SERIAL "-R";

static const struct usb_string iphone_strings[] = {
    {.id = 1, .s = "Apple Inc."},
    {.id = 2, .s = "iPhone"},
    /* iPhone uses 24 real chars + 16 null bytes at the end; this is not supported by libcomposite (it reads the strings to first null-byte) */
    /* at this point no headunit seems that strict in caring about this */
    {.id = 3, .s = iphone_serial_buf},
    /* this one has no trailing zero bytes and is a copy of the serial with -R at the end */
    {.id = 4, .s = iphone_serial_r_buf},
    {.id = 5, .s = "PTP"},
    {.id = 6, .s = "iPod USB Interface"},
    {.id = 7, .s = "PTP + Apple Mobile Device"},
    {.id = 8, .s = "PTP + Apple Mobile Device + Apple USB Ethernet"},
    {.id = 9, .s = "PTP + Apple Mobile Device + NCM"},
    {.id = 10, .s = "PTP + Apple Mobile Device + Valeria"},
    {.id = 11, .s = "Apple Mobile Device + Interdevice Audio"},
    {.id = 12, .s = "Interdevice Audio Interfaces"},
    {.id = 13, .s = "PTP + Apple Mobile Device + Apple USB Ethernet + NCM"},
    {.id = 14, .s = "NCM Direct Only"},
    {.id = 15, .s = "C605EEC903D4"},
    {.id = 16, .s = "Apple USB Multiplexor"},
    {.id = 17, .s = "82B9896BEB4D"},
    {.id = 18, .s = "NCM Control"},
    {.id = 19, .s = "NCM Control"},
    {.id = 20, .s = "AppleUSBEthernet"},
    {.id = 21, .s = "Valeria"},
    {.id = 22, .s = "PTP"},
    {.id = 23, .s = "IDAM MIDI Streaming Interface"},
    {.id = 24, .s = "82B9896BEB4D"},
    {.id = 25, .s = "NCM Control Direct"},
    {.id = 26, .s = "NCM Data"},
    {.id = 27, .s = "NCM Data"},
    {}};

/*static struct usb_gadget_strings iphone_stringtab = {
    .language = 0x0409,
    .strings = iphone_strings,
};
static struct usb_gadget_strings *iphone_strings_array[] = {
    &iphone_stringtab,
    NULL,
};*/
