# Lossless Scaling: what it does, and whether it helps this Mac

Lossless Scaling (developer and publisher "THS", Steam app 993090, $6.99) adds frame generation
("LSFG") and spatial upscaling to games that do not have them. It works **outside the game**: it
captures the game's window and shows a processed copy on top of it. This page is a **web and docs
research pass only** (2026-11-07). It is a paid Steam app, so no binary was bought, downloaded or
inspected, nothing was run, and no part of it was decompiled. (The Linux port lsfg-vk is open
source but is a Vulkan layer, not the Windows app, so it says nothing about the Windows app's
capture path or anti-cheat behaviour.)

Each claim is marked **[verified]** (official Steam store page or the developer's Steam
announcements), **[community]** (users, reviewers, community guides), **[repo]** (this project's
own measurements) or **[unknown]**.

## Short version

- **How it works:** a separate process grabs the game's window through Windows' own screen-capture
  APIs (Desktop Duplication or Windows Graphics Capture) and shows generated or upscaled frames in
  its own window on top. Nothing is officially said about injecting into the game, and the design
  doesn't need it. The developer doesn't promise it is anti-cheat safe.
- **Cost:** LSFG runs on the GPU. On a GPU that is already at its power limit, it takes watts and
  time from the game, so the real ("base") frame rate goes down. The developer says to cap the game
  so the GPU is not at 100 %.
- **Latency:** frame generation holds back one real frame. Expect about one base-frame time on top,
  plus a few ms: roughly +10–25 ms at 40–60 base FPS on fast GPUs, more when the GPU is saturated.
- **BF6/Javelin:** EA has said nothing. There is one EA forum report of an account ban while using
  it (cause not confirmed) and one unanswered "is it allowed?" thread. **Risk: unknown, not zero.**
- **On this Mac: not worth it.** The internal display is 60 Hz with no VRR, so frame generation
  can't show more than 60 FPS. The GPU is power-limited, and BF6 is a fast online shooter where
  added latency matters. The iGPU can't act as a second GPU. See the adoption items at the end.

## 1. How it works

**Capture [verified].** The developer's announcements describe a "new DXGI frame capture API" (Feb
2024), a "capture engine overhaul" with a *Queue Target* setting (Mar 2025), and Windows 11 24H2
changes to "the Desktop Duplication API (DXGI)" that make frame pacing depend on MPO
(Multiplane Overlay) support ([Steam news feed](https://api.steampowered.com/ISteamNews/GetNewsForApp/v2/?appid=993090&count=15&maxlength=1500)).
The app's FAQ site, which links to the official Reddit, Discord and Steam pages, lists both
**DXGI Desktop Duplication** and **Windows Graphics Capture (WGC)** as capture options. It
recommends WGC on 24H2 and DXGI on older builds
([FAQ: DXGI vs WGC](https://sageinfinity.github.io/docs/FAQ/dxgiwgc)) [community/semi-official].
Both are ordinary user-mode OS APIs, the same ones OBS and Discord use for screen capture.

**Presentation [verified in effect, details community].** The app draws into its own window over
the game. The store page says it supports windowed and borderless games, and that exclusive
fullscreen "may work only if the output is set to a second display"
([Steam store](https://store.steampowered.com/app/993090/)). That only makes sense if a separate
window sits on top of the game. There are its own *Sync mode* (VSync / allow tearing) and *Max
Frame Latency* settings for its swap chain (LSFG 2.2 notes, Jul 2024), and it renders the cursor
itself at the output rate. It stops drawing frames above the refresh rate (LSFG 2.1 notes): **the
output is capped at the monitor's refresh rate.**

**Injection or hooking [unknown, likely none].** No official statement either way. Users say it
"doesn't directly hook into any API" and "doesn't interact with the game API or pipelines"
([Steam thread](https://steamcommunity.com/app/993090/discussions/0/4286937147059426029)), but
none of them is the developer, and none of them shows evidence. The design (capture the window,
show it in another window, and the second-display advice for exclusive fullscreen) only works if it
does **not** inject. An injected tool would hook `Present` and work in exclusive fullscreen too.
The FAQ recommends running it **as administrator** for games with anti-cheat, so it can capture and
draw over elevated game windows (same FAQ page). Running elevated is a small thing, but it is one
more reason an anti-cheat might look at the process.

**What LSFG is [verified].** A "proprietary machine learning model" doing frame interpolation on
the GPU ([store](https://store.steampowered.com/app/993090/)). Main versions (all from the Steam
news feed above):

| Version | Date | What changed |
|---|---|---|
| LSFG 1.0 | 2024-01 | First release; "can run on integrated graphics" |
| LSFG 2.0 | 2024-05 | Better quality, 1.5–2× the GPU load; "Performance" mode keeps 1.x cost |
| LSFG 2.1 / 2.3 | 2024-06 / 08 | X3 and X4 multipliers; X3 ≈ 1.7× the load of X2 |
| LSFG 3.0 | 2025-01-10 | ≈40 % less GPU load at X2 than LSFG 2, up to X20, "Resolution Scale" (flow scale) |
| Adaptive FG | 2025-03 | Fractional multiplier to hit a target FPS; Queue Target 0/1/2 |
| LSFG 3.1 | 2025-06 | Quality fixes; new Performance mode "up to 2× GPU load reduction" |
| Hotfix | 2025-07-14 | Performance mode quality fix (newest announcement as of 2026-11-07) |

**Upscalers [verified].** Spatial only: LS1 (its own), AMD FSR (1), NVIDIA NIS, SGSR v1, plus
Integer/XBR for pixel art and Anime4K ([store](https://store.steampowered.com/app/993090/)). A
spatial upscaler sees only the finished frame, HUD included. BF6 has its own **temporal**
upscalers (FSR, XeSS, DLSS) that run before the HUD is drawn
([TweakTown](https://www.tweaktown.com/news/106785/battlefield-6-to-support-nvidia-dlss-amd-fsr-intel-xess-upscaling-frame-generation-at-launch/index.html)).
For BF6, Lossless Scaling's upscaling has nothing to add.

## 2. GPU cost on a weak, power-capped GPU

**Official guidance [verified].** "Free GPU resources are preferred" (store page). LSFG 3 notes:
"locking the game framerate is recommended. This helps to avoid 100 % GPU load (reducing its
impact on latency) and ensures smoother framepacing." The base frame rate should be at least 30 FPS
at 1080p (40 preferred, 60 ideal). Lower flow scale ("Resolution Scale") cuts cost: 90 % ≈ the old
Performance mode, 75 % suggested for 1440p, 50 % for 4K. LSFG 3.1's Performance mode claims up to
2× less load.

**RDNA1 / RX 5500-class figures [community, thin].** No measured RX 5500M or RX 5500 XT number was
found. A community tier list places the RX 5500 XT as a *dedicated second GPU* able to do X2 frame
generation to 120 Hz output at 2160p
([r/OptimizedGaming LSFG resource](https://redlib.ssps.io/r/OptimizedGaming/comments/1jisc76/ultimate_lsfg_resource/mjpqu4v/?context=3)).
That is a full desktop card doing nothing else. Users with single GPUs report that "the base fps
hit is substantial" (RX 5700 XT owner) and that the base drop "only happens at 100 % GPU usage"
([Steam](https://steamcommunity.com/app/993090/discussions/0/4355624856143493185)). Adaptive mode
costs about 11 % more than fixed X2 for the same output (same r/OptimizedGaming guide).

**This machine [repo + reasoning].** The 5500M sits at its firmware power limit (30–35 W) the whole
match. It gives ~60 FPS at 35 W and ~41–48 FPS at 30 W ([hardware-test](../hardware-test.md)). The
GPU can't speed up to absorb extra work, so whatever LSFG costs, the game loses. LSFG at X2,
Performance mode and a 50–75 % flow scale is cheap compared to BF6 itself, maybe a few percent of
the frame time, but **it is not free**. Its load is also counted against the same shared CPU+GPU
budget. **It hurts the base frame rate here; the only question is how much** [unknown: needs a test].

**Dual GPU [verified that it exists; community on details].** The store page lists dual-GPU support
("may reduce latency"). The community guide says how it works: the render GPU copies each real frame
over PCIe to a second GPU, which generates the extra frames **and drives the display**. The display
should be connected to the second GPU, or the frames have to be copied back
([dual GPU guide](https://sageinfinity.github.io/docs/Guides/DualGPUGuide)). Laptop iGPUs vary a lot;
something like a Radeon 780M is recommended above 1080p, and a UHD 620 owner reports it was "not
powerful enough"
([Steam/techpowerup reports via search](https://steamcommunity.com/app/993090/discussions/0/594016978256007124)).

**The Mac's Intel UHD 630 as that second GPU: no [community + repo context].**
- On 2018–2019 MacBook Pros the EFI turns the Intel GPU off when anything but macOS boots. It is
  not in Device Manager under Boot Camp
  ([gpu-switch README](https://github.com/0xbb/gpu-switch),
  [monitortests forum](https://www.monitortests.com/forum/Thread-MacBook-Pro-2019-adding-FreeSync-range-possible)).
- It can be woken with `apple_set_os.efi`, but users report that the internal display can't
  then be driven from the iGPU ([egpu.io build log](https://egpu.io/forums/builds/2019-16-macbook-pro-rp5500m-9th8ch-rx-5700-xt-32gbps-tb3-rog-xg-station-2-macos-10-15-1-win10-1903-itsage/)).
  The guide wants the display on the second GPU.
- The iGPU sits inside the CPU package, so its watts come from the same shared ~50 W firmware
  budget that already starves the 5500M. It would take watts from the GPU it is meant to help.
- A UHD 630 is about as weak as the UHD 620 that users say can't keep up.

Changing EFI boot behaviour also falls outside what gamebooster does: reversible Windows tweaks.

## 3. Added input latency

Frame interpolation has to wait for the next real frame before it can show the one in between. So
the floor is about **one base-frame time** on top of normal latency, plus processing time.

- **Developer [verified]:** LSFG 3 has "about 24 % better end-to-end latency" than LSFG 2, measured
  with OSLTT at 40 base FPS, X2 (LSFG 3 notes). Locking the frame rate below 100 % GPU load reduces
  latency. *Queue Target 0* is the lowest-latency capture mode. X3 adds no latency over X2.
- **Independent LSFG 3 test [community, via Tom's Hardware]:** 40 FPS base 28.5 ms → LSFG 3 X2
  (80 FPS) 41.7 ms, i.e. **+13 ms**; LSFG 2 was 54.3 ms
  ([Tom's Hardware](https://tomshardware.com/video-games/pc-gaming/lossless-scaling-3-update-touts-greatly-improved-latency-and-performance-universal-frame-gen-tool-boasts-24-percent-reduced-latency)).
- **PC Games Hardware, LDAT, LSFG 2-era (May 2024) [community, measured]:** Cyberpunk 4K on an
  RTX 4080 Super: native ~39 ms, DLSS 3 FG ~61 ms, LSFG ~90 ms (Reflex on). On an RX 7900 XTX:
  native ~45 ms → LSFG ~80 ms (Anti-Lag on). On a GTX 1080 Ti at ~41 FPS base: ~72 ms → LSFG
  ~115 ms. CS2 capped at 65 FPS: ~22–25 ms → ~41–48 ms
  ([PCGH, p. 2](https://www.pcgameshardware.de/Starfield-Spiel-61756/Specials/Lossless-Scaling-Test-Frame-Generation-FSR-3-DLSS-Latenz-LSFG-1446068/2/)).
  These numbers come from a saturated GPU with an older LSFG, the case closest to this Mac.
- **User tools at 60 base FPS [community, weak]:** +5 ms (X2) to +10–13 ms (X4), or 29–32 →
  36–42 ms. The methods vary.

**For this Mac:** at a 30 FPS base (what X2 to 60 Hz needs, see below), one base frame is 33 ms.
Expect roughly **+35–45 ms**, about double, on a GPU that is already saturated. In an online shooter
you would feel it.

## 4. BF6, Javelin and other anti-cheats

**EA / BF6 [unknown; one unconfirmed ban report].**
- EA has said nothing about Lossless Scaling. EA's help pages say Javelin "will shut down the game
  … if you use cheat tools, debuggers, or other conflicting software" and that EA keeps a
  denylist of software and drivers ([EA anticheat help](https://help.ea.com/en-ca/help/pc/ea-anticheat)).
  The pages found don't say whether capture or overlay tools are on the denylist.
- **Ban report:** "Random account ban after using Lossless Scaling". The player says the account
  was deactivated for violating the EA User Agreement after using it for frame generation, with no
  earlier bans. A moderator pointed them to an appeal. The thread is marked solved, but what solved
  it isn't visible, and whether the ban was overturned is unknown
  ([EA forum](https://forums.ea.com/discussions/battlefield-6-technical-issues-en/random-account-ban-after-using-lossless-scaling/12741924);
  the page returned 403 here, so this summary comes from search excerpts).
- **Clarification request:** a player asked EA Help directly. The agents said EA "doesn't
  recommend third-party software that may affect gameplay" and could not say whether Lossless
  Scaling counts, or whether it ever caused false bans
  ([EA forum](https://forums.ea.com/discussions/battlefield-6-general-discussion-en/request-for-clarification-is-lossless-scaling-safe-to-use-with-ea-anticheat/12780739),
  also from search excerpts).
- Nothing credible was found either way: no controlled test, no list of people who used it
  without trouble, no ban wave. One unproven ban report on its own doesn't show a detection, but it
  is more than any other boost tool researched here has against it.

**Developer stance [verified].** The store page says: "the community has found it safe to use with
online games that have built-in anti-cheat systems. However, this cannot be guaranteed and should
be used with user discretion." That's a hedge, not a promise. One Steam user says a game's support
advised against it on their platform; another warns it is "NOT guaranteed safe" in Blizzard games
([Steam thread](https://steamcommunity.com/app/993090/discussions/0/4418676609604606777)).

**EAC, BattlEye, Vanguard [community only].** Users report using it with "many EAC games, nguard
games, even CoD Ricochet" without problems
([Steam thread](https://steamcommunity.com/app/993090/discussions/0/4286937147059426029)). No
statement from Epic, BattlEye or Riot was found. Claims elsewhere that "EAC, BattlEye and Vanguard
don't flag it" come from spam pages and are ignored here.

**Risk level for BF6 on this account: low-to-moderate and unquantified.** The technique (a
separate process using OS capture APIs, no injection) is the same as screen recorders, which
anti-cheats generally allow. But an elevated process captures the game window and draws over it
all match long, which looks a lot like an ESP overlay. EA won't clear it, and one ban report exists.
Javelin is kernel-level, and a ban is not undone by uninstalling.

## 5. Requirements and settings

- **Window mode [verified]:** windowed or borderless. Exclusive fullscreen works only with output
  on a second display.
- **OS / GPU [verified]:** minimum Windows 10 2004 and "modern integrated graphics", DirectX 11.
  Recommended Windows 11 24H2 with RTX 30 / RX 6000 / Arc. The RX 5500M (RDNA1) is below the
  recommended tier.
- **Frame cap [verified]:** recommended, to keep the GPU below 100 %. In fixed mode, cap at
  refresh ÷ multiplier, e.g. 1/3 of refresh for X3 (2.1 notes).
- **Base FPS [verified]:** at least 30 at 1080p (40 preferred, 60 ideal); at least 40 at 1440p.
- **Output [verified]:** capped at the display refresh rate.
- **VRR [verified]:** G-Sync support since 2.3, improved FreeSync on AMD since 2.2. The guide
  says Enhanced Sync, Fast Sync and Adaptive Sync in the driver don't work with it
  ([r/OptimizedGaming guide](https://lr.eu.psf.lt/r/OptimizedGaming/comments/1jjl1t0/ultimate_lossless_scaling_upscaling_resource)) [community].
- **Windows 11 24H2+ [verified/community]:** DXGI capture needs MPO; WGC is suggested instead.
  This machine runs 26H2, so it would be **WGC**.
- **Community "low latency" settings:** Queue Target 0, Max Frame Latency 1–3 (3 suggested for
  AMD), sync off (allow tearing), overlays off.

## Adoption items for gamebooster-rs

1. **Not a boost step; nothing to build.** It's a separate capture-and-present app, not a Windows
   setting. gamebooster shouldn't install, launch or configure it.
2. **Verdict for this Mac: don't use it for BF6.** Three reasons, any one enough:
   - **60 Hz internal panel, no VRR** ([repo](../measuring.md)). The output can't go above 60 FPS,
     so X2 means capping BF6 at **30 FPS**. That's the bottom of the developer's minimum, and it
     roughly doubles input latency (+35–45 ms) to show a 60 FPS image the GPU nearly reaches on
     its own at 35 W. Adaptive mode toward 60 from a ~45 FPS base fills only 15 frames a second,
     costs more GPU than fixed mode, and still adds about one base frame (~22 ms) of latency.
   - **The GPU is at its power limit.** LSFG's cost comes straight out of the base frame rate (and
     the shared CPU+GPU budget). There is no free GPU time to spend.
   - **Anti-cheat risk is unknown, not zero** (one EA forum ban report, no EA clearance) on a paid
     account, all for a smoothness gain that this panel mostly can't show.
   Its spatial upscalers are also worse than BF6's own FSR/XeSS.
3. **If you try it anyway (single-player or offline games without anti-cheat, not BF6):**
   borderless window, capture API **WGC** (26H2), LSFG 3.1 **X2 fixed** with **Performance mode
   on** and flow scale **50–75 %**, Queue Target 0, Max Frame Latency 3, sync on (60 Hz, no VRR).
   Cap the game at **30 FPS** with its own limiter, and only in slow-paced games where 30 FPS
   input feels fine. It is worth more on an **external 120/144 Hz monitor**, with the game capped
   at 60 (if the 5500M can hold 60 there) for X2 → 120. Measure the real base frame rate in
   the game's own overlay. In BF6, ETW tools are blocked anyway ([measuring](../measuring.md)).
4. **Better first for BF6:** the game's own upscaler (FSR or XeSS Quality/Balanced) to get more
   real frames from the 30–35 W budget. FSR 3 frame generation inside the game is the engine-native
   alternative. It is EA-shipped, so there's no anti-cheat question, and FSR 3 FG normally runs on
   RDNA1. Whether BF6 offers it on the 5500M is [unknown]: check its graphics menu. The same 60 Hz
   and latency limits apply.
5. **Dual GPU with the Intel UHD 630: not realistic.** Firmware hides it under Boot Camp, waking it
   needs EFI tricks, the panel stays on the 5500M, the iGPU shares the same power budget, and it is
   too weak anyway.
