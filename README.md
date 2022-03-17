# stl-organizer

## Project setup
Follow the [Tauri Setup Instructions](https://tauri.studio/en/docs/getting-started/setup-windows/) to install rust, build tools, yarn, etc.  

On Windows (not WSL), make sure the Microsoft Visual Studio C++ **2019** build tools are installed and the 2017 version aren't.  Having 2017 installed can lead to `This version of %1 is not compatible with the version of Windows you’re running` when running the application.  Build tools are available [here](https://visualstudio.microsoft.com/downloads/).

### Development
Run the application with hot reloading with: `yarn tauri dev`

### License
Copyright 2021 John Hungerford.

The stl-organizer application is licensed under the [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0).

The repository includes files with different licenses:
* `src-tauri/test/resources/Benchy.zip` - #3DBenchy, by CreativeTools.se.  Downloaded from [thingiverse](https://www.thingiverse.com/thing:763622), licensed under [`Creative Commons - Attribution - No Derivatives`](https://creativecommons.org/licenses/by-nd/4.0/)
* `src-tauri/test/resources/frighterbench-v2.stl` - Frighter benchy, by Matt (@LiveIn3D).  Downloaded from [My Mini Factory](https://www.myminifactory.com/object/3d-print-the-freighter-benchy-84304), licensed under `MyMiniFactory Exclusive - Credit - Remix - Noncommercial`.