use sdl2::image::InitFlag;

pub struct SdlSetup {
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub timer_subsystem: sdl2::TimerSubsystem,
    pub ttf_context: sdl2::ttf::Sdl2TtfContext,
}

pub fn init() -> SdlSetup {
    let sdl_context: sdl2::Sdl = sdl2::init().unwrap();
    let video_subsystem: sdl2::VideoSubsystem = sdl_context.video().unwrap();
    let timer_subsystem = sdl_context.timer().unwrap();
    let ttf_context = sdl2::ttf::init().expect("Failed to init true type fonts");
    sdl2::image::init(InitFlag::PNG | InitFlag::JPG).unwrap();

    SdlSetup {
        sdl_context,
        video_subsystem,
        timer_subsystem,
        ttf_context,
    }
}
