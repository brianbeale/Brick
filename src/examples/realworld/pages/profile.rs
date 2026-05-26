use super::super::api::{self, ArticlesResponse, Profile};
use super::super::auth::AuthStore;
#[allow(unused_imports)]
use super::super::routes::RwRoute;
use crate::view_components::*;

#[model]
pub struct ProfilePage {
    #[prop]
    pub username: String,
    #[default(Load::Idle)]
    pub profile: Load<Profile>,
    #[default(Load::Idle)]
    pub articles: Load<ArticlesResponse>,
    #[default(false)]
    pub show_favorited: bool,
    #[default(0u32)]
    pub page: u32,
    #[default(false)]
    pub logged_in: bool,
}

#[controller]
impl ProfilePage {
    #[on(mount)]
    pub fn init(&mut self) {
        let auth = AuthStore::get();
        self.logged_in.set(!auth.token.read().is_empty());
        self.load_profile();
        self.load_articles();
    }

    pub fn show_my_articles(&mut self) {
        self.show_favorited.set(false);
        self.page.set(0);
        self.load_articles();
    }

    pub fn show_favorited_articles(&mut self) {
        self.show_favorited.set(true);
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

    pub fn toggle_follow(&mut self) {
        let auth = AuthStore::get();
        let token = auth.token.read();
        if token.is_empty() {
            return;
        }
        let username = self.username.clone();
        let profile = self.profile.clone();
        let token2 = token.clone();
        #[cfg(brick_dom)]
        spawn_local(async move {
            let following = if let Load::Loaded(ref p) = profile.read() {
                p.following
            } else {
                false
            };
            let result = if following {
                api::unfollow_user(&token2, &username).await
            } else {
                api::follow_user(&token2, &username).await
            };
            if let Ok(p) = result {
                profile.set(Load::Loaded(p));
            }
        });
    }
}

impl ProfilePage {
    fn load_profile(&self) {
        let username = self.username.clone();
        let profile = self.profile.clone();
        let auth = AuthStore::get();
        let token = auth.token.read();
        let token_opt = if token.is_empty() {
            None
        } else {
            Some(token.clone())
        };
        #[cfg(brick_dom)]
        spawn_local(async move {
            profile.set(Load::Loading);
            match api::get_profile(&username, token_opt.as_deref()).await {
                Ok(p) => profile.set(Load::Loaded(p)),
                Err(e) => profile.set(Load::Failed(e)),
            }
        });
    }

    fn load_articles(&self) {
        let username = self.username.clone();
        let favorited = self.show_favorited.read();
        let page = self.page.read();
        let limit = 10u32;
        let offset = page * limit;
        let articles = self.articles.clone();
        let auth = AuthStore::get();
        let token = auth.token.read();
        let token_opt = if token.is_empty() {
            None
        } else {
            Some(token.clone())
        };
        #[cfg(brick_dom)]
        spawn_local(async move {
            articles.set(Load::Loading);
            match api::get_profile_articles(
                &username,
                token_opt.as_deref(),
                favorited,
                limit,
                offset,
            )
            .await
            {
                Ok(r) => articles.set(Load::Loaded(r)),
                Err(e) => articles.set(Load::Failed(e)),
            }
        });
    }
}

#[view(ProfilePage)]
fn render() -> Box<ViewComposite> {
    style! {
        .profile-page { max-width: 960px; margin: 0 auto; padding: 0 1rem; }
        .profile-header { background: #f3f3f3; padding: 2rem 1rem; text-align: center; margin-bottom: 1.5rem; }
        .profile-header img { width: 100px; height: 100px; border-radius: 50%; margin-bottom: 0.75rem; }
        .profile-header h4 { font-size: 1.25rem; font-weight: 700; margin-bottom: 0.25rem; }
        .profile-header p { color: #aaa; margin-bottom: 0.75rem; }
        .follow-btn { padding: 0.3rem 0.75rem; border: 1px solid #aaa; background: none; color: #aaa; cursor: pointer; border-radius: 3px; font-size: 0.9rem; }
        .profile-tabs { display: flex; border-bottom: 2px solid #ddd; margin-bottom: 1.5rem; }
        .profile-tab { padding: 0.5rem 1rem; background: none; border: none; border-bottom: 2px solid transparent; cursor: pointer; color: #aaa; margin-bottom: -2px; }
        .profile-tab-active { padding: 0.5rem 1rem; background: none; border: none; border-bottom: 2px solid #5cb85c; cursor: pointer; color: #5cb85c; margin-bottom: -2px; }
        .article-preview { border-top: 1px solid #ddd; padding: 1.5rem 0; }
        .article-meta { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 1rem; }
        .article-meta img { width: 32px; height: 32px; border-radius: 50%; }
        .author-info { display: flex; flex-direction: column; }
        .author-name { font-weight: 600; color: #5cb85c; text-decoration: none; }
        .article-date { color: #bbb; font-size: 0.8rem; }
        .fav-btn { margin-left: auto; padding: 0.25rem 0.6rem; border: 1px solid #5cb85c; border-radius: 3px; background: none; color: #5cb85c; cursor: pointer; font-size: 0.85rem; }
        .read-more { color: #bbb; font-size: 0.85rem; text-decoration: none; }
        .pagination { display: flex; gap: 0.5rem; justify-content: center; margin-top: 1rem; }
        .status-msg { color: #aaa; padding: 1rem 0; }
        .error-msg { color: #e53e3e; padding: 1rem 0; }
    }
    children! {
        div {
            class("profile-page"),
            catch!(my.profile, |err| p(&format!("Error: {}", err.read())).c("error-msg"),
                wait!(my.profile, p("Loading profile…").c("status-msg"),
                    load!(my.profile, |prof| {
                        let avatar_src = prof.image.clone().unwrap_or_default();
                        let bio = prof.bio.clone().unwrap_or_default();
                        let follow_label = if prof.following { "- Unfollow" } else { "+ Follow" };
                        Box::new(crate::view_components::BrickContainer {
                            class: "profile-header",
                            children: vec![
                                img().attr("src", &avatar_src).attr("alt", &prof.username).into_component(),
                                h4(&prof.username).into_component(),
                                p(&bio).into_component(),
                                button(follow_label).c("follow-btn").trigger(&my.toggle_follow).into_component(),
                            ],
                        })
                    }),
                ),
            ),
            div {
                class("profile-tabs"),
                button("My Articles").c("profile-tab").trigger(&my.show_my_articles),
                button("Favorited Articles").c("profile-tab").trigger(&my.show_favorited_articles),
            },
            catch!(my.articles, |err| p(&format!("Error: {}", err.read())).c("error-msg"),
                wait!(my.articles, p("Loading articles…").c("status-msg"),
                    load!(my.articles, |resp| {
                        let mut items: Vec<Box<dyn crate::view_components::Brick>> = resp.articles.iter().map(|art| {
                            let author_href = format!("/@{}", art.author.username);
                            let article_href = format!("/article/{}", art.slug);
                            let avatar_src = art.author.image.clone().unwrap_or_default();
                            let created = art.created_at[..10].to_string();
                            let fav_count = art.favorites_count;
                            Box::new(crate::view_components::BrickContainer {
                                class: "article-preview",
                                children: vec![
                                    div {
                                        class("article-meta"),
                                        img().attr("src", &avatar_src).attr("alt", &art.author.username),
                                        div {
                                            class("author-info"),
                                            a(&art.author.username).c("author-name").attr("href", &author_href),
                                            span(&created).c("article-date"),
                                        },
                                        button(&format!("♥ {}", fav_count)).c("fav-btn"),
                                    }.into_component(),
                                    h2(&art.title).into_component(),
                                    p(&art.description).into_component(),
                                    a("Read more…").c("read-more").attr("href", &article_href).into_component(),
                                ],
                            }) as Box<dyn crate::view_components::Brick>
                        }).collect();
                        if items.is_empty() {
                            items.push(p("No articles yet.").c("status-msg").into_component());
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::cascade;

    fn page() -> ProfilePage {
        ProfilePage {
            username: "alice".to_string(),
            ..cascade()
        }
    }

    #[test]
    fn renders_tabs() {
        let html = crate::view_components::render_to_html(&*page().into_component());
        assert!(html.contains("My Articles"));
        assert!(html.contains("Favorited Articles"));
    }

    #[test]
    fn initial_profile_idle() {
        assert!(matches!(page().profile.read(), Load::Idle));
    }

    #[test]
    fn initial_not_favorited_tab() {
        assert!(!page().show_favorited.read());
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
    fn show_favorited_sets_flag() {
        let mut p = page();
        p.show_favorited_articles();
        assert!(p.show_favorited.read());
    }

    #[test]
    fn show_my_articles_clears_flag() {
        let mut p = page();
        p.show_favorited.set(true);
        p.show_my_articles();
        assert!(!p.show_favorited.read());
    }
}
