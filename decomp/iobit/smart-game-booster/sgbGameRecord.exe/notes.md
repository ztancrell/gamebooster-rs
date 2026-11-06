# sgbGameRecord.exe notes (encoder)

Carved copy: `pe_046fda05_2666496.bin` (2,666,496 bytes, Delphi). Built on FFVCL Pro 7.4 (a Delphi
wrapper around FFmpeg; `LibAV\avcodec-58.dll`, avformat-58, avfilter-7, swscale-5, swresample-3 =
FFmpeg 4.x). Reads the shared-memory ring buffers and runs an FFmpeg command line built from:
`-f frameinput -input_type dc|canvas -pix_fmt bgr24 -r %d -s %dX%d` (video),
`-f waveinput -ac %d -ar %d -sample_fmt flt|s16` (audio), `-c:v mpeg4 -q %d`.
So: software MPEG-4 Part 2 encoding at a fixed quality, no GPU encoder (the avcodec build contains
h264_nvenc/h264_amf/libx264/libopenh264, but this command line does not use them). Falls back to
GDI `BitBlt` capture ("BitBlt failed"). Logs `%LOCALAPPDATA%\PCGameBoost\Smart Game Booster\Logs\
VideoLogger_exe.log`. No network imports.
