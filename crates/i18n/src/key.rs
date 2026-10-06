macro_rules! keys {
    ($($name:ident),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Key {
            $($name),*
        }

        impl Key {
            pub const ALL: &'static [Key] = &[$(Key::$name),*];
        }
    };
}

keys! {
    AppSubtitle,
    NavLibrary, NavVideos, NavMusic, NavImages, NavPlaylist,
    SectionScreens, AllScreens, ScreensDetected, ScreenName, PrimaryScreen,
    EngineTitle, EngineActive, EngineStopped, PlaybackTitle, PlaybackPaused, PlaybackPlaying,
    SearchHint, Import, SettingsTooltip, MuteSound, UnmuteSound, PausePlayback, ResumePlayback,
    NowPlaying, NothingPlaying, EngineMemory, LastMinute, EngineOff, ActiveScreens, Muted,
    PlaylistOn, PlaylistOff, NoResults, NoResultsHint, EmptyTitle, EmptyBody, ImportMedia,
    ImportShortcut,
    CardActive, AddToPlaylist, RemoveFromPlaylist, RemoveFromLibrary,
    KindVideo, KindAudio, KindImage,
    Customize, Close, Apply, Remove, Reset, SectionPlayback, Volume, Speed, Loop, Scaling,
    Visualizer, SectionColors, Brightness, Contrast, Saturation, Gamma,
    ScaleFill, ScaleFit, ScaleStretch, ScaleCenter, VisOff, VisWaves, VisSpectrum,
    PolicyContinue, PolicyMute, PolicyPause, OrderSequential, OrderShuffle,
    QualityPerformance, QualityBalanced, QualityHigh,
    SettingsTitle, SettingsSubtitle, CardLanguage, CardLanguageDesc, CardAudio, CardAudioDesc,
    MasterVolume, MasterVolumeDesc, MuteAll, CardPerformance, CardPerformanceDesc, LimitFps,
    LimitFpsDesc, FramesPerSecond, RenderQuality, RenderQualityDesc, HardwareDecoding,
    HardwareDecodingDesc, CardEnergy, CardEnergyDesc, AppFocused, AppMaximized, AppFullscreen,
    OnlyThisScreen, OnBattery, OnBatteryDesc, CardPlaylist, CardPlaylistDesc, AutoRotation,
    Interval, Order, CardSystem, CardSystemDesc, Autostart, AutostartDesc, RenderEngine,
    EngineRunning, EngineHalted, StopEngine, StartEngine, UnitMinutes, UnitMegabytes,
    DropTitle, DropBody,
    ToastAddedToPlaylist, ToastRemovedFromPlaylist, ToastApplied, ToastRemoved,
    ToastEngineStarted, ToastEngineStopped, ToastImported, ToastImportedPartial, ToastNoScreen,
    ErrUnsupported, ErrMissing, ErrRange, ErrNotFound, ErrDuplicate, ErrStorage, ErrSystem,
    ErrRenderer,
    TrayOpen, TrayPause, TrayMute, TrayQuit,
    StartupFailed,
}
