# tf2-launcher

## Requirements
Rust 1.65 or greater (for generic associated type needed by iced_graphics).  
Minimal supported version for OSX: 10.8 (see .cargo/config.toml, also .github/rust.yml)  

## Inspiration UI
https://www.artstation.com/artwork/3qrn9o

## Official team fortress palette
https://lospec.com/palette-list/team-fortress-2-official

## TF2 Web API
https://wiki.teamfortress.com/wiki/WebAPI

## Todo
- [ ] Images for each server according to the map played

- [ ] Get timeout with retry in case of failure
whenever I get the information. But be carefull as querying Teamwork.tf too often might cause issues.

- [ ] Cancel reload? Progresive loading: instead of waiting for the whole reload process I could update the list of servers
- [ ] Display progression while reloading?
