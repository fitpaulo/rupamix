# Rust Pulse Audio Mixer

This started a project to test my Rust chops after going through the rust book. It uses [libpulse-binding](https://github.com/jnqnfe/pulse-binding-rust)
to drive the interadtion with the Pulse server. It currently offers enough features to bind to keypresses
for increasing, decreasing, and muting the default sink (speakers).

## Example Usage For Defualt Sink
To increase the audio by the defualt 5:
    `rupamix i`
    
To increase the audio by N:
    `rupamix i --increment N`

To decrease the audio py the defualt 5:
    `rupamix d`
    
To decrease the audio by N:
    `rupamix d --increment N`

## You can specify the sink as well
To get the list of sink indexes and names:
    `rupamix p -v`

            Index -- Name                                             
    --------------------------------------------------------------------
              146 -- alsa_output.pci-0000_0b_00.4.iec958-stereo       
    (default) 150 -- alsa_output.usb-Audient_iD4-00.HiFi__hw_iD4__sink
              315 -- alsa_output.pci-0000_09_00.1.hdmi-stereo-extra2  

Then update a sink like the following:

`rupamix -i 146 i --increment 10`

OR

`rupamix -n alsa_output.pci-0000_0b_00.4.iec958-stereo i -i 10`

## Toggle Mute
It can toggle mute as well: 
    `rupamix t`


