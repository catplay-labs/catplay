// SPDX-License-Identifier: GPL-2.0
#pragma once
#include <uapi/linux/usb/audio.h>

static DECLARE_UAC_AC_HEADER_DESCRIPTOR(1)
cs_intf1 = {
    .bLength = UAC_DT_AC_HEADER_SIZE(1),
    .bDescriptorType = USB_DT_CS_INTERFACE,
    .bDescriptorSubtype = UAC_HEADER,
    .bcdADC = cpu_to_le16(0x0100),
    .wTotalLength = cpu_to_le16(0x001e),
    .bInCollection = 1,
    .baInterfaceNr = {1},
};

static struct uac_input_terminal_descriptor cs_intf2 = {
    .bLength = UAC_DT_INPUT_TERMINAL_SIZE,
    .bDescriptorType = USB_DT_CS_INTERFACE,
    .bDescriptorSubtype = UAC_INPUT_TERMINAL,
    .bTerminalID = 1,
    .wTerminalType = cpu_to_le16(0x0201),
    .bAssocTerminal = 2,
    .bNrChannels = 2,
    .wChannelConfig = cpu_to_le16(0x0003),
    .iChannelNames = 0,
    .iTerminal = 0,
};

static struct uac1_as_header_descriptor cs_intf4 = {
    .bLength = UAC_DT_AS_HEADER_SIZE,
    .bDescriptorType = USB_DT_CS_INTERFACE,
    .bDescriptorSubtype = UAC_AS_GENERAL,
    .bTerminalLink = 2,
    .bDelay = 1,
    .wFormatTag = cpu_to_le16(UAC_FORMAT_TYPE_I_PCM),
};

DECLARE_UAC_FORMAT_TYPE_I_DISCRETE_DESC(9);

static struct uac1_output_terminal_descriptor cs_intf3 = {
    .bLength = UAC_DT_OUTPUT_TERMINAL_SIZE,
    .bDescriptorType = USB_DT_CS_INTERFACE,
    .bDescriptorSubtype = UAC_OUTPUT_TERMINAL,
    .bTerminalID = 2,
    .wTerminalType = cpu_to_le16(0x0101),
    .bAssocTerminal = 1,
    .bSourceID = 1,
    .iTerminal = 0,
};

static struct uac_format_type_i_discrete_descriptor_9 cs_intf5 = {
    .bLength = UAC_FORMAT_TYPE_I_DISCRETE_DESC_SIZE(9),
    .bDescriptorType = USB_DT_CS_INTERFACE,
    .bDescriptorSubtype = UAC_FORMAT_TYPE,
    .bFormatType = UAC_FORMAT_TYPE_I,
    .bNrChannels = 2,
    .bSubframeSize = 2,
    .bBitResolution = 16,
    .bSamFreqType = 9,
    .tSamFreq = {
        {0x40, 0x1F, 0x00}, /*  8000 */
        {0x11, 0x2B, 0x00}, /* 11025 */
        {0xE0, 0x2E, 0x00}, /* 12000 */
        {0x80, 0x3E, 0x00}, /* 16000 */
        {0x22, 0x56, 0x00}, /* 22050 */
        {0xC0, 0x5D, 0x00}, /* 24000 */
        {0x00, 0x7D, 0x00}, /* 32000 */
        {0x44, 0xAC, 0x00}, /* 44100 */
        {0x80, 0xBB, 0x00}, /* 48000 */
    },
};
