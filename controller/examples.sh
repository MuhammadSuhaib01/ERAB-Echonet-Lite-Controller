#!/bin/bash
# Example scripts for ECHONET Controller automation

# This script demonstrates various ways to control ECHONET Lite devices programmatically

# Example 1: Basic light control script
# Usage: ./light_control.sh on|off <device_index> <object_code>

set -e

show_usage() {
    echo "ECHONET Light Control Script"
    echo "Usage: $0 <command> [args...]"
    echo ""
    echo "Commands:"
    echo "  turn_on <dev> <obj>           Turn on device"
    echo "  turn_off <dev> <obj>          Turn off device"
    echo "  set_brightness <dev> <obj> <0-255>  Set brightness level"
    echo "  read_status <dev> <obj>       Read device status"
    echo "  discover                       Discover all devices"
    echo ""
    echo "Examples:"
    echo "  $0 discover"
    echo "  $0 turn_on 1 029101"
    echo "  $0 turn_off 1 029101"
    echo "  $0 set_brightness 1 029101 128"
}

if [ $# -eq 0 ]; then
    show_usage
    exit 1
fi

COMMAND=$1

case $COMMAND in
    discover)
        echo "Searching for ECHONET Lite devices..."
        docker run -it --network host echonet_controller << EOF
search
list
exit
EOF
        ;;
    
    turn_on)
        if [ $# -lt 3 ]; then
            echo "Usage: $0 turn_on <device_index> <object_code>"
            exit 1
        fi
        DEV=$2
        OBJ=$3
        docker run -it --network host echonet_controller << EOF
write $DEV $OBJ 80 30
exit
EOF
        ;;
    
    turn_off)
        if [ $# -lt 3 ]; then
            echo "Usage: $0 turn_off <device_index> <object_code>"
            exit 1
        fi
        DEV=$2
        OBJ=$3
        docker run -it --network host echonet_controller << EOF
write $DEV $OBJ 80 31
exit
EOF
        ;;
    
    set_brightness)
        if [ $# -lt 4 ]; then
            echo "Usage: $0 set_brightness <device_index> <object_code> <0-255>"
            exit 1
        fi
        DEV=$2
        OBJ=$3
        BRIGHTNESS=$4
        # Convert decimal to hex
        HEX_BRIGHTNESS=$(printf "%02x" $BRIGHTNESS)
        docker run -it --network host echonet_controller << EOF
write $DEV $OBJ b0 $HEX_BRIGHTNESS
exit
EOF
        ;;
    
    read_status)
        if [ $# -lt 3 ]; then
            echo "Usage: $0 read_status <device_index> <object_code>"
            exit 1
        fi
        DEV=$2
        OBJ=$3
        docker run -it --network host echonet_controller << EOF
read $DEV $OBJ 80
exit
EOF
        ;;
    
    *)
        echo "Unknown command: $COMMAND"
        show_usage
        exit 1
        ;;
esac
