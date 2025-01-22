pub mod templates;

use actix_web::{
    dev::ResourcePath,
    get,
    http::header::{
        AcceptEncoding, ContentEncoding, Encoding, Header, IfNoneMatch, Preference,
        CONTENT_ENCODING, CONTENT_TYPE, ETAG,
    },
    web, App, HttpRequest, HttpResponse, HttpServer,
};
use embed_it::{Blake3_256Hash, Entry, EntryPath, Index, StrContent};
use hex::ToHex;
use templates::{DirModel, EntryModel, Templates};
use tinytemplate::TinyTemplate;
use tracing_subscriber::{filter, layer::SubscriberExt, Layer, Registry};

#[derive(embed_it::Embed)]
#[embed(
    path = "$CARGO_MANIFEST_DIR/public",
    dir(
        derive(Blake3),
        field(factory = ETagHeaderValue, name = etag, trait_name = DirETagField, global),
        field(factory = DirHtml, name = html, global),
    ),
    file(
        derive(Blake3),
        derive(Zstd),
        derive(Gzip),
        derive(Brotli),
        field(factory = ETagHeaderValue, name = etag, trait_name = FileETagField, global),
    )
)]
pub struct Public;

pub struct ETagHeaderValue {
    header: String,
    value: String,
}

impl ETagHeaderValue {
    pub fn create<T: Blake3_256Hash + ?Sized>(v: &T) -> Self {
        let value = v.blake3_256().encode_hex();
        Self { header: format!("\"{value}\""), value }
    }
}

impl DirFieldFactory for ETagHeaderValue {
    type Field = Self;

    fn create<T: Dir + ?Sized>(data: &T) -> Self::Field {
        Self::create(data)
    }
}

impl FileFieldFactory for ETagHeaderValue {
    type Field = Self;

    fn create<T: File + ?Sized>(data: &T) -> Self::Field {
        Self::create(data)
    }
}

pub struct DirHtml;

impl DirFieldFactory for DirHtml {
    type Field = String;

    fn create<T: Dir + ?Sized>(data: &T) -> Self::Field {
        let model = DirModel {
            title: data.path().name(),
            entries: data.entries().iter().map(|e| {
                e.map(|d| EntryModel {
                    href: d.path().relative_path_str(),
                    title: d.path().name(),
                    icon_src: Public.icons().dir().path().relative_path_str(),
                }, |f| EntryModel {
                    href: f.path().relative_path_str(),
                    title: f.path().name(),
                    icon_src: Public.icons().file().path().relative_path_str(),
                }).value()
            }).collect(),
        };

        let mut tt = TinyTemplate::new();
        tt.add_template("dir", Templates.dir().str_content()).unwrap();
        tt.render("dir", &model).unwrap()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ChoosenEncoding {
    Zstd,
    Brotli,
    Gzip,
    #[default]
    Identity,
}

impl ChoosenEncoding {
    pub fn content<F: File + ?Sized>(&self, file: &F) -> &'static [u8] {
        match self {
            ChoosenEncoding::Zstd => file.zstd_content(),
            ChoosenEncoding::Brotli => file.brotli_content(),
            ChoosenEncoding::Gzip => file.gzip_content(),
            ChoosenEncoding::Identity => file.content(),
        }
    }

    pub fn find_first(mut header: AcceptEncoding) -> Option<ChoosenEncoding> {
        header.0.sort_by(|l, r| r.quality.cmp(&l.quality));
        header
            .0
            .into_iter()
            .filter_map(|v| match v.item {
                Preference::Any => None,
                Preference::Specific(e) => match e {
                    Encoding::Known(e) => match e {
                        ContentEncoding::Identity => Some(Self::Identity),
                        ContentEncoding::Brotli => Some(Self::Brotli),
                        ContentEncoding::Deflate => None,
                        ContentEncoding::Gzip => Some(Self::Gzip),
                        ContentEncoding::Zstd => Some(Self::Zstd),
                        _ => todo!(),
                    },
                    Encoding::Unknown(_) => None,
                },
            })
            .next()
    }

    pub fn encoding(&self) -> ContentEncoding {
        match self {
            ChoosenEncoding::Zstd => ContentEncoding::Zstd,
            ChoosenEncoding::Brotli => ContentEncoding::Brotli,
            ChoosenEncoding::Gzip => ContentEncoding::Gzip,
            ChoosenEncoding::Identity => ContentEncoding::Identity,
        }
    }
}

#[get("/public/{tail:.*}")]
async fn public(request: HttpRequest, tail: web::Path<String>) -> HttpResponse {
    tracing::info!("request: {tail:?}");
    match Public.get(tail.path().trim_end_matches('/')) {
        Some(entry) => {
            let etag = entry.map(|d| d.etag(), |f| f.etag()).value();
            let is_match = match IfNoneMatch::parse(&request) {
                Err(e) => {
                    tracing::info!("Unable to parse if-none-match header: {e:?}");
                    false
                },
                Ok(IfNoneMatch::Any) => return HttpResponse::NotModified().finish(),
                Ok(IfNoneMatch::Items(tags)) => {
                    tags.into_iter().find(|v| v.tag() == &etag.value).is_some()
                }
            };

            if is_match {
                return HttpResponse::NotModified().finish();
            }

            match entry {
                Entry::File(f) => {
                    let chosen_encoding = AcceptEncoding::parse(&request)
                        .ok()
                        .and_then(ChoosenEncoding::find_first)
                        .unwrap_or_default();
                    let content = chosen_encoding.content(f.into_file());
                    let encoding = chosen_encoding.encoding();
                    HttpResponse::Ok()
                        .insert_header((CONTENT_ENCODING, encoding))
                        .insert_header((ETAG, etag.header.as_str()))
                        .body(content)
                }
                Entry::Dir(d) => {
                    let html = d.html().as_bytes();
                    HttpResponse::Ok()
                        .insert_header((CONTENT_ENCODING, ContentEncoding::Identity))
                        .insert_header((CONTENT_TYPE, "text/html"))
                        .body(html)
                }
            }
        }
        None => HttpResponse::NotFound().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = Registry::default().with(
        tracing_subscriber::fmt::layer()
            .pretty()
            .with_ansi(true)
            .with_filter(filter::LevelFilter::from_level(tracing::Level::DEBUG)),
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();
    
    HttpServer::new(|| App::new().service(public))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
