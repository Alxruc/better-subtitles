# Better Subtitles for YouTube
Leverage local transcription models through Whisper to generate your own YouTube subtitles for any video, because YouTube's subtitle generation is sometimes inaccurate (especially for languages that are not English). 

**This is not a finished application, do not expect it to be bug-free**

This repo comes with the main Tauri application to run a lightweight GUI to view all your transcriptions and transcribe using a Rust backend and a partner browser extension to easily pass along links with the click of a button.

## Required installs
The program uses Vulcan in order to accelerate the model using any GPU architecture.

Example Fedora install
```
sudo dnf install vulkan-loader vulkan-loader-devel mesa-vulkan-drivers vulkan-headers vulkan-validation-layers mesa-dri-drivers vulkan-tools
```

Example Arch install
```
sudo pacman -Syu vulkan-icd-loader vulkan-devel vulkan-radeon vulkan-headers vulkan-validation-layers mesa vulkan-tools
```

Also needs ffmpeg and yt-dlp. Make sure to keep yt-dlp updated!

## Launching the application
This is still in development and is **not** finished yet at all, but it can be tested using the `bun run tauri` commands. 

## How does it work?
1. Load a *GGML* Whisper model (fine-tuned for your target language)
The app can save multiple models just download them and then load them via the file browser.
2. Add the browser extension to your browser of choice (since it is still in development it has to be added through debug options)
3. Go to the video you want to transcribe and open the browser extension with the program running in the background and click "Transcribe"
4. Wait for the model to finish and then click "Get Subtitles"

The subtitles should now be displayed on your video