# Asus-control-center

ASUS laptop control center for Arch Linux, built specifically for my own device. It provides a convenient interface for managing and monitoring ASUS-specific hardware features, system settings, performance profiles, and other laptop controls that may not be available through standard Linux tools.


## Project Plan:

### 1. Hardware Information

This stage of the project will focus on creating a reliable way to monitor the current state of the laptop.

The application will initially operate primarily as a read-only dashboard, displaying useful information about the system and its hardware.

Depending on the available Linux interfaces and hardware support, the application will display information such as:

* CPU model and usage
* CPU temperature
* GPU information and usage
* Which GPU is currently active
* GPU temperature
* Fan RPM
* Current fan or thermal mode
* Battery charge level
* Battery charging state
* Battery charge limit
* Other relevant hardware information exposed by the system

The implementation would use existing Linux interfaces and system facilities wherever possible instead of accessing hardware directly. The exact information available will depend on the hardware and drivers of the target laptop.

### 2. ASUS-Specific Controls

Once hardware detection and monitoring are working reliably, the project will be extended with functionality for controlling ASUS-specific hardware features.

The application would use appropriate Linux interfaces, drivers, and system services wherever possible.

The exact functionality will depend on what the laptop's hardware and Linux drivers expose.

Potential controls:

* Keyboard backlight on/off
* Keyboard backlight brightness
* RGB lighting, if supported
* Lighting effects
* Lighting speed
* Manual fan curves, if safely supported
* Fan speed monitoring
* Temperature-based fan control
* Integrated/dedicated GPU mode
* GPU power state
* GPU performance settings
* GPU monitoring
* GPU power limits, where supported

Not every feature is expected to be available on the target device.

The main goal of this stage is to turn the monitoring functionality from the first stage into a practical control interface for the hardware features that can be safely managed through Linux.


### 3. Profiles

After the individual hardware controls have been implemented, the next stage will introduce configurable profiles.

Profiles will provide a way to combine several hardware and system settings into a single configuration. Instead of manually changing individual settings, the user will be able to select a profile representing a particular usage scenario.

Example profiles:

#### Quiet

Possible settings:

* Lower performance limits
* Quieter fan curve
* Lower GPU power usage
* Reduced keyboard lighting
* Battery-saving settings

#### Balanced

A general-purpose configuration:

* Normal CPU performance
* Automatic fan control
* Normal GPU behavior
* Moderate power consumption

#### Performance

Possible settings:

* Higher CPU performance
* More aggressive fan curve
* Maximum performance GPU mode
* Higher performance power settings
* Full keyboard lighting

#### Custom Profiles

Users would be able to create their own profiles.

For example:

> "Gaming"

could configure:

* Performance mode
* Dedicated GPU
* Aggressive fan curve
* Keyboard RGB enabled
* Higher performance power settings

### 4. CLI and GUI

The project will initially benefit from a command-line interface for testing and interacting with the underlying functionality.

The CLI will make it possible to test hardware detection and control independently of the graphical interface. This will also provide a simple way to verify that individual features work correctly before they are integrated into the GUI.

Once the underlying hardware-control functionality is stable, the project will receive a graphical user interface designed to provide a more convenient experience.

The GUI will act primarily as a frontend for the existing hardware and profile functionality.

The GUI may provide:

* Hardware monitoring
* Fan and temperature information
* Performance mode selection
* GPU controls
* Keyboard and RGB controls
* Profile selection
* Custom profile management
* Notifications for unsupported features

The final GUI will depend on which functionality is successfully implemented during the earlier stages of the project.

### 5. Broader ASUS Compatibility

The initial version of the project will be developed specifically for my own ASUS laptop. This is intended to keep the scope manageable while allowing the hardware-specific functionality to be properly investigated and tested.

After the application works reliably on my device, the project can be extended toward supporting other ASUS laptops.