# tf2-launcher [![Test](https://github.com/IohannRabeson/teamwork-launcher/actions/workflows/test.yml/badge.svg)](https://github.com/IohannRabeson/teamwork-launcher/actions/workflows/test.yml)

Launcher for Team Fortress 2 that uses Teamwork.tf as data source.  

![Main view screenshot](/screenshots/main_view.png?raw=true)

# How to clone
Mind to also clone sub modules with `--recursive`:
`git clone --recursive https://github.com/IohannRabeson/teamwork-launcher.git`

# How to use it
You must have a [Teamwork](https://teamwork.tf) API key.  
To get one, connect to [teamwork.tf](https://teamwork.tf), go to https://teamwork.tf/settings and click "Show optional settings".  

You must copy/paste your key in the settings page.  
Alternatively you can specify an environment variable `TEAMWORK_API_KEY`.

# Testing mode
The testing mode force the application to store the configuration and caches in a temporary directory.  
To enable this mode, pass the flag `--testing-mode`.