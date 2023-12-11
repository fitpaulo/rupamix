# Rust Pulse Audio Mixer

This started a project to test my Rust chops after going through the rust book. It uses [libpulse-binding](https://github.com/jnqnfe/pulse-binding-rust)
to drive the interadtion with the Pulse server. It currently offers enough features to bind to keypresses
for increasing, decreasing, and muting the default sink (speakers).

## Install
### Use Cargo

    cargo install rupamix --profile optimized

### Manual

    git clone git@github.com:fitpaulo/rupamix.git \
    && cd rupamix \
    && cargo build --profile optimized
    
### Make it visible to your WM/Desktop
To use it with your window manger, i.e. bind it to a keyboard press, your window manager needs to see the binary.
The easiest way is to copy the execsuteale into /usr/bin (this assumes you installed with cargo)

    sudo cp $HOME/.cargo/bin/rupamix /usr/bin/.

Or you can link it:

    sudo ln -sf $HOME/.cargo/bin/rupamix /usr/bin/.
## Usage examples
To increase the audio by the defualt 5:
    
    rupamix increase
    
To increase the audio by N:
    
    rupamix increase --increment <N>

To decrease the audio py the defualt 5:
    
    rupamix decrease
    
To decrease the audio by N:
    
    rupamix decrease --increment <N>

To set the volume directly

    rupamix set --volume 120 --boost 

## You can specify the sink as well
To get the list of sink indexes and names:
    
    rupamix print --volume

            Index -- Name                                             
    --------------------------------------------------------------------
              146 -- alsa_output.pci-0000_0b_00.4.iec958-stereo       
    (default) 150 -- alsa_output.usb-Audient_iD4-00.HiFi__hw_iD4__sink
              315 -- alsa_output.pci-0000_09_00.1.hdmi-stereo-extra2  

Then update a sink like the following:

    rupamix --index 146 increase --increment 10

OR

    rupamix --name alsa_output.pci-0000_0b_00.4.iec958-stereo increase --increment 10

## Toggle Mute
It can toggle mute as well: 
    
    rupamix t


