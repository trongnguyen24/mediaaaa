#!/bin/bash
mkdir -p src-tauri/binaries
cd src-tauri/binaries

echo "Downloading yt-dlp..."
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos -o yt-dlp-aarch64-apple-darwin
chmod +x yt-dlp-aarch64-apple-darwin

echo "Downloading ffmpeg..."
curl -L https://evermeet.cx/ffmpeg/ffmpeg-7.1.zip -o ffmpeg.zip
unzip -o ffmpeg.zip
mv ffmpeg ffmpeg-aarch64-apple-darwin
chmod +x ffmpeg-aarch64-apple-darwin
rm ffmpeg.zip

echo "Binaries downloaded successfully."
