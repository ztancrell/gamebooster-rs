# VideoRecord64.dll / VideoRecord.dll notes (frame hand-off)

Carved copies: `VideoRecord64.dll` `pe_0413a53d_2257408.bin`, `VideoRecord.dll` `pe_045aea05_1372160.bin`.
Delphi DLL loaded into the game by FPS64.dll. Exports `AddBMP`, `AddAudio`, `Picture2Video`,
`SavePicDataToFile`, `StopCapture`. Starts `sgbGameRecord.exe` with `CreateProcessW` and pushes
frames/audio into named shared memory `SMGB_GAME_RECORD_4.3_VIDEO_*` / `SMGB_GAME_RECORD_4.3_AUDIO_*`
(CreateFileMapping/MapViewOfFile, mutex). No encoding here, no network.
