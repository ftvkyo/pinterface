# `pinterface`

## Obsidian setup

```sh
# Dependencies:

sudo apt install \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libgtk-3-0 \
    libgbm1

# Optional:

sudo apt install \
    libgl1 \
    libx11-xcb1 \
    libegl1 \

# Virtual screen:

sudo apt install xvfb
```

### `/etc/systemd/system/obsidian-sync.service`

```ini
[Unit]
Description=Obsidian Sync

[Service]
User=ftvkyo
Group=ftvkyo
ExecStart=/usr/bin/xvfb-run /home/ftvkyo/bin/obsidian

[Install]
WantedBy=multi-user.target
```

Then:

```sh
sudo systemctl enable --now obsidian-sync
```


## SPI setup

### Config

```sh
sudo raspi-config
```

Then: enable SPI interface

### `/boot/cmdline.txt`

Add this parameter to allow long messages to be sent at once:

```
spidev.bufsiz=1024000
```
