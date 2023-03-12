# tf2-launcher [![Test](https://github.com/IohannRabeson/teamwork-launcher/actions/workflows/test.yml/badge.svg)](https://github.com/IohannRabeson/teamwork-launcher/actions/workflows/test.yml)

Launcher for Team Fortress 2 that uses Teamwork.tf as data source.  

![Main view red](/screenshots/main_view_red.png?raw=true)
![Main view blu](/screenshots/main_view_blu.png?raw=true)

# How to clone
Mind to also clone sub modules with `--recursive`:
`git clone --recursive https://github.com/IohannRabeson/teamwork-launcher.git`

# Supported platforms
The application is tested on Windows and partially tested on MacOS, and might build and run on Linux.  
Pull requests are welcome to improve specific platform support.

On MacOS (and Linux), the ping can't be queried without starting the application with privileges, this is a limitation
that comes from the library surge-ping (see [#30](https://github.com/kolapapa/surge-ping/issues/30)).

# How to use it
You must have a [Teamwork](https://teamwork.tf) API key.  
To get one, connect to [teamwork.tf](https://teamwork.tf), go to https://teamwork.tf/settings and click "Show optional settings".  

You must copy/paste your key in the settings page.  
Alternatively you can specify an environment variable `TEAMWORK_API_KEY`.

# Testing mode
The testing mode force the application to store the configuration and the thumbnails cache in a temporary directory, so the application
writes files but the temporary directory is deleted when the application quits and any existing configuration is untouched.  
To enable this mode, pass the flag `--testing-mode`.  