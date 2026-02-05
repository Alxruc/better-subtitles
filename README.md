# Better Subtitles for YouTube
Leverage local transcription models through Whisper to generate your own YouTube subtitles for any video, because YouTube's subtitle generation is sometimes inaccurate (especially for languages that are not English). 

**This is not a finished application, do not expect it to be bug-free**

This repo comes with the main Tauri application to run a lightweight GUI to view all your transcriptions and transcribe using a Rust backend and a partner browser extension to easily pass along links with the click of a button.

## Required installs
The program uses Vulcan in order to accelerate the model using any GPU architecture.

Example Fedora install
```
sudo dnf install vulkan-loader mesa-vulkan-drivers vulkan-headers vulkan-validation-layers mesa-dri-drivers vulkan-tools
```

Example Arch install
```
sudo pacman -Syu vulkan-icd-loader vulkan-radeon vulkan-headers vulkan-validation-layers mesa vulkan-tools
```

Also needs ffmpeg and yt-dlp. Make sure to keep yt-dlp updated!

## Launching the application
Using `bun run tauri` or `npm run tauri` but this is early in development so don't do it