#make_bin#
#LOAD_SEGMENT=0100#
#LOAD_OFFSET=0100#
#CS=0100#
#IP=0100#
#DS=0100#
#ES=0100#
#SS=0100#
#SP=FFFE#
org 100h
mov ax, 7FFFh
add ax, 1
mov [0200h], ax
hlt
