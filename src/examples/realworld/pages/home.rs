use super::super::api::{self, ArticlesResponse};
use super::super::auth::AuthStore;
#[allow(unused_imports)]
use super::super::routes::RwRoute;
use crate::view_components::*;

#[model]
pub struct HomePage {
    #[default(Load::Idle)]
    pub articles: Load<ArticlesResponse>,
    #[default(Load::Idle)]
    pub tags: Load<Vec<String>>,
    #[default("global".to_string())]
    pub active_tab: String,
    #[default(0u32)]
    pub page: u32,
    #[default(String::new())]
    pub selected_tag: String,
    #[default(false)]
    pub logged_in: bool,
}

#[controller]
impl HomePage {
    #[on(mount)]
    pub fn init(&mut self) {
        let auth = AuthStore::get();
        self.logged_in.set(!auth.token.read().is_empty());
        self.load_articles();
        let tags = self.tags.clone();
        #[cfg(brick_dom)]
        spawn_local(async move {
            tags.set(Load::Loading);
            match api::get_tags().await {
                Ok(t) => tags.set(Load::Loaded(t)),
                Err(e) => tags.set(Load::Failed(e)),
            }
        });
    }

    pub fn show_global(&mut self) {
        self.active_tab.set("global".to_string());
        self.selected_tag.set(String::new());
        self.page.set(0);
        self.load_articles();
    }

    pub fn show_feed(&mut self) {
        self.active_tab.set("feed".to_string());
        self.selected_tag.set(String::new());
        self.page.set(0);
        self.load_articles();
    }

    pub fn load_tag(&mut self) {
        let tag = self.selected_tag.read();
        if tag.is_empty() {
            return;
        }
        self.active_tab.set(format!("#{}", tag));
        self.page.set(0);
        self.load_articles();
    }

    pub fn prev_page(&mut self) {
        let p = self.page.read();
        if p > 0 {
            self.page.set(p - 1);
            self.load_articles();
        }
    }

    pub fn next_page(&mut self) {
        let p = self.page.read();
        self.page.set(p + 1);
        self.load_articles();
    }
}

impl HomePage {
    pub fn load_articles(&self) {
        let tab = self.active_tab.read();
        let page = self.page.read();
        let limit = 10u32;
        let offset = page * limit;
        let articles = self.articles.clone();
        let auth = AuthStore::get();
        let token = auth.token.read();

        if tab == "feed" {
            let token2 = token.clone();
            #[cfg(brick_dom)]
            spawn_local(async move {
                articles.set(Load::Loading);
                match api::get_feed(&token2, limit, offset).await {
                    Ok(r) => articles.set(Load::Loaded(r)),
                    Err(e) => articles.set(Load::Failed(e)),
                }
            });
        } else {
            let tag = if tab.starts_with('#') {
                Some(tab[1..].to_string())
            } else {
                None
            };
            #[cfg(brick_dom)]
            spawn_local(async move {
                articles.set(Load::Loading);
                match api::get_articles(tag.as_deref(), None, None, limit, offset).await {
                    Ok(r) => articles.set(Load::Loaded(r)),
                    Err(e) => articles.set(Load::Failed(e)),
                }
            });
        }
    }
}

#[view(HomePage)]
fn render() -> Box<ViewComposite> {
    style! {
        .home-banner { background: #5cb85c; color: white; padding: 2rem 1rem; text-align: center; margin-bottom: 1.5rem; }
        .home-banner h1 { font-size: 2.5rem; font-weight: 700; margin-bottom: 0.25rem; letter-spacing: -1px; }
        .home-banner p { font-size: 1.1rem; opacity: 0.9; }
        .home-layout { display: flex; gap: 1.5rem; max-width: 960px; margin: 0 auto; padding: 0 1rem; }
        .article-list { flex: 1; min-width: 0; }
        .sidebar { width: 220px; flex-shrink: 0; }
        .feed-tabs { display: flex; gap: 0; border-bottom: 2px solid #ddd; margin-bottom: 1rem; }
        .feed-tab { padding: 0.5rem 1rem; background: none; border: none; border-bottom: 2px solid transparent; cursor: pointer; color: #aaa; margin-bottom: -2px; }
        .feed-tab-active { padding: 0.5rem 1rem; background: none; border: none; border-bottom: 2px solid #5cb85c; cursor: pointer; color: #5cb85c; margin-bottom: -2px; }
        .article-preview { border-top: 1px solid #ddd; padding: 1.5rem 0; }
        .article-meta { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 1rem; }
        .avatar { width: 32px; height: 32px; border-radius: 50%; }
        .author-info { display: flex; flex-direction: column; }
        .author-name { font-weight: 600; color: #5cb85c; text-decoration: none; }
        .article-date { color: #bbb; font-size: 0.8rem; }
        .fav-wrap { margin-left: auto; }
        .fav-btn { padding: 0.25rem 0.6rem; border: 1px solid #5cb85c; border-radius: 3px; background: none; color: #5cb85c; cursor: pointer; font-size: 0.85rem; }
        .article-preview h2 { margin-bottom: 0.25rem; font-size: 1.25rem; }
        .article-preview p { color: #999; margin-bottom: 0.75rem; }
        .read-more { color: #bbb; font-size: 0.85rem; text-decoration: none; }
        .pagination { display: flex; gap: 0.5rem; justify-content: center; margin-top: 1rem; }
        .tag-cloud { background: #f3f3f3; border-radius: 4px; padding: 0.75rem; }
        .tag-cloud h4 { margin-bottom: 0.5rem; font-size: 0.9rem; }
        .tag-list { display: flex; flex-wrap: wrap; gap: 0.35rem; }
        .tag-pill { padding: 0.2rem 0.5rem; background: #818a91; color: white; border-radius: 10px; font-size: 0.8rem; cursor: pointer; border: none; }
        .status-msg { color: #aaa; padding: 1rem 0; }
        .error-msg { color: #e53e3e; padding: 1rem 0; }
    }
    let auth = AuthStore::get();
    let logged_in = !auth.token.read().is_empty();
    children! {
        div {
            class("home-banner"),
            h1("conduit"),
            p("A place to share your knowledge."),
        },
        div {
            class("home-layout"),
            div {
                class("article-list"),
                div {
                    class("feed-tabs"),
                    when!(my.logged_in,
                        button("Your Feed").c("feed-tab").trigger(&my.show_feed),
                        span("").c(""),
                    ),
                    button("Global Feed").c("feed-tab").trigger(&my.show_global),
                },
                catch!(my.articles, |err| p(&format!("Error: {}", err.read())).c("error-msg"),
                    wait!(my.articles, p("Loading articles…").c("status-msg"),
                        load!(my.articles, |resp| {
                            let mut items: Vec<Box<dyn crate::view_components::Brick>> = resp.articles.iter().map(|art| {
                                let author_href = format!("/@{}", art.author.username);
                                let article_href = format!("/article/{}", art.slug);
                                let fav_count = art.favorites_count;
                                let avatar_src = art.author.image.clone().unwrap_or_default();
                                let created = art.created_at[..10].to_string();
                                Box::new(crate::view_components::BrickContainer {
                                    class: "article-preview",
                                    children: vec![
                                        div {
                                            class("article-meta"),
                                            img().attr("src", &avatar_src).attr("alt", &art.author.username).c("avatar"),
                                            div {
                                                class("author-info"),
                                                a(&art.author.username).c("author-name").attr("href", &author_href),
                                                span(&created).c("article-date"),
                                            },
                                            div {
                                                class("fav-wrap"),
                                                button(&format!("♥ {}", fav_count)).c("fav-btn"),
                                            },
                                        }.into_component(),
                                        h2(&art.title).into_component(),
                                        p(&art.description).into_component(),
                                        a("Read more…").c("read-more").attr("href", &article_href).into_component(),
                                    ],
                                }) as Box<dyn crate::view_components::Brick>
                            }).collect();
                            if items.is_empty() {
                                items.push(p("No articles found.").c("status-msg").into_component());
                            }
                            Box::new(crate::view_components::BrickContainer {
                                class: "",
                                children: items,
                            })
                        }),
                    ),
                ),
                div {
                    class("pagination"),
                    button("← Prev").trigger(&my.prev_page),
                    button("Next →").trigger(&my.next_page),
                },
            },
            div {
                class("sidebar"),
                div {
                    class("tag-cloud"),
                    h4("Popular Tags"),
                    catch!(my.tags, |_err| p("Could not load tags.").c("status-msg"),
                        wait!(my.tags, p("Loading…").c("status-msg"),
                            load!(my.tags, |tags| {
                                Box::new(crate::view_components::BrickContainer {
                                    class: "tag-list",
                                    children: tags.iter().map(|tag| -> Box<dyn crate::view_components::Brick> {
                                        button(tag.as_str()).c("tag-pill")
                                    }).collect(),
                                })
                            }),
                        ),
                    ),
                },
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::cascade;

    fn page() -> HomePage {
        HomePage { ..cascade() }
    }

    #[test]
    fn renders_banner() {
        assert!(crate::view_components::render_to_html(&*page().into_component()).contains("conduit"));
    }

    #[test]
    fn initial_articles_idle() {
        assert!(matches!(page().articles.read(), Load::Idle));
    }

    #[test]
    fn initial_tab_is_global() {
        assert_eq!(page().active_tab.read(), "global");
    }

    #[test]
    fn initial_page_is_zero() {
        assert_eq!(page().page.read(), 0);
    }

    #[test]
    fn prev_page_no_underflow() {
        let mut p = page();
        p.prev_page();
        assert_eq!(p.page.read(), 0);
    }

    #[test]
    fn next_page_increments() {
        let mut p = page();
        p.next_page();
        assert_eq!(p.page.read(), 1);
    }

    #[test]
    fn show_global_resets_tab() {
        let mut p = page();
        p.active_tab.set("feed".to_string());
        p.show_global();
        assert_eq!(p.active_tab.read(), "global");
        assert_eq!(p.page.read(), 0);
    }

    #[test]
    fn show_feed_sets_tab() {
        let mut p = page();
        p.show_feed();
        assert_eq!(p.active_tab.read(), "feed");
    }
}
