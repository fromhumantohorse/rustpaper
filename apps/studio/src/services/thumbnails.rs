use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};
use rustpaper_application::ports::ThumbnailRenderer;
use rustpaper_domain::wallpaper::{Wallpaper, WallpaperId};
use rustpaper_infrastructure::branding::decode_png;

struct Job {
    id: WallpaperId,
    source: PathBuf,
    target: PathBuf,
}

struct Rendered {
    id: WallpaperId,
    image: Option<ColorImage>,
}

pub struct Thumbnails {
    directory: PathBuf,
    jobs: Sender<Job>,
    results: Receiver<Rendered>,
    textures: HashMap<WallpaperId, TextureHandle>,
    requested: HashSet<WallpaperId>,
}

impl Thumbnails {
    pub fn spawn(directory: PathBuf, renderer: Box<dyn ThumbnailRenderer>, ctx: Context) -> Self {
        let (jobs, queue) = mpsc::channel::<Job>();
        let (publish, results) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("thumbnails".into())
            .spawn(move || {
                for job in queue {
                    let image = load_or_render(renderer.as_ref(), &job);
                    if publish.send(Rendered { id: job.id, image }).is_err() {
                        return;
                    }
                    ctx.request_repaint();
                }
            });
        if let Err(error) = spawned {
            log::error!("générateur de miniatures indisponible : {error}");
        }
        Self {
            directory,
            jobs,
            results,
            textures: HashMap::new(),
            requested: HashSet::new(),
        }
    }

    pub fn request(&mut self, wallpapers: &[Wallpaper]) {
        for wallpaper in wallpapers {
            if !self.requested.insert(wallpaper.id()) {
                continue;
            }
            let job = Job {
                id: wallpaper.id(),
                source: wallpaper.source().path().to_path_buf(),
                target: self.file_for(wallpaper.id()),
            };
            if self.jobs.send(job).is_err() {
                log::debug!("file des miniatures fermée");
            }
        }
    }

    pub fn collect(&mut self, ctx: &Context) {
        for rendered in self.results.try_iter() {
            if !self.requested.contains(&rendered.id) {
                remove_thumbnail(&self.directory, rendered.id);
                continue;
            }
            let Some(image) = rendered.image else {
                continue;
            };
            let name = format!("thumbnail-{}", rendered.id);
            let texture = ctx.load_texture(name, image, TextureOptions::LINEAR);
            self.textures.insert(rendered.id, texture);
        }
    }

    pub fn textures(&self) -> &HashMap<WallpaperId, TextureHandle> {
        &self.textures
    }

    pub fn forget(&mut self, id: WallpaperId) {
        self.textures.remove(&id);
        self.requested.remove(&id);
        remove_thumbnail(&self.directory, id);
    }

    fn file_for(&self, id: WallpaperId) -> PathBuf {
        thumbnail_file(&self.directory, id)
    }
}

fn thumbnail_file(directory: &Path, id: WallpaperId) -> PathBuf {
    directory.join(format!("{id}.png"))
}

fn remove_thumbnail(directory: &Path, id: WallpaperId) {
    if let Err(error) = fs::remove_file(thumbnail_file(directory, id)) {
        log::debug!("miniature #{id} non supprimée : {error}");
    }
}

fn load_or_render(renderer: &dyn ThumbnailRenderer, job: &Job) -> Option<ColorImage> {
    if !job.target.is_file() {
        if let Some(parent) = job.target.parent() {
            fs::create_dir_all(parent).ok()?;
        }
        if let Err(error) = renderer.render(&job.source, &job.target) {
            log::info!("miniature #{} : {error}", job.id);
            return None;
        }
    }
    decode(&job.target)
}

fn decode(path: &Path) -> Option<ColorImage> {
    let bytes = fs::read(path).ok()?;
    let image = decode_png(&bytes).ok()?;
    let size = [
        usize::try_from(image.width).ok()?,
        usize::try_from(image.height).ok()?,
    ];
    Some(ColorImage::from_rgba_unmultiplied(size, &image.rgba))
}
