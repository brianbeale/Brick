use super::super::api::{self, Article, Comment};
use super::super::auth::AuthStore;
use super::super::routes::{RwRoute, navigate};
use crate::view_components::*;

#[model]
pub struct ArticlePage {
    #[prop]
    pub slug: String,
    #[default(Load::Idle)]
    pub article: Load<Article>,
    #[default(Load::Idle)]
    pub comments: Load<Vec<Comment>>,
    #[default(String::new())]
    pub new_comment: String,
    #[default(String::new())]
    pub error: String,
    #[default(false)]
    pub logged_in: bool,
}

#[controller]
impl ArticlePage {
    #[on(mount)]
    pub fn init(&mut self) {
        let auth = AuthStore::get();
        self.logged_in.set(!auth.token.read().is_empty());
        let slug = self.slug.clone();
        let article = self.article.clone();
        let comments = self.comments.clone();
        let slug2 = slug.clone();
        #[cfg(brick_dom)]
        spawn_local(async move {
            article.set(Load::Loading);
            match api::get_article(&slug).await {
                Ok(a) => article.set(Load::Loaded(a)),
                Err(e) => article.set(Load::Failed(e)),
            }
        });
        #[cfg(brick_dom)]
        spawn_local(async move {
            comments.set(Load::Loading);
            match api::get_comments(&slug2).await {
                Ok(c) => comments.set(Load::Loaded(c)),
                Err(e) => comments.set(Load::Failed(e)),
            }
        });
    }

    pub fn submit_comment(&mut self) {
        let body = self.new_comment.read();
        if body.trim().is_empty() {
            return;
        }
        let auth = AuthStore::get();
        let token = auth.token.read();
        if token.is_empty() {
            return;
        }
        let slug = self.slug.clone();
        let comments = self.comments.clone();
        let new_comment = self.new_comment.clone();
        let token2 = token.clone();
        let body2 = body.clone();
        #[cfg(brick_dom)]
        spawn_local(async move {
            if api::add_comment(&token2, &slug, &body2).await.is_ok() {
                new_comment.set(String::new());
                match api::get_comments(&slug).await {
                    Ok(c) => comments.set(Load::Loaded(c)),
                    Err(e) => comments.set(Load::Failed(e)),
                }
            }
        });
    }

    pub fn delete_article(&mut self) {
        let auth = AuthStore::get();
        let token = auth.token.read();
        if token.is_empty() {
            return;
        }
        let slug = self.slug.clone();
        let token2 = token.clone();
        #[cfg(brick_dom)]
        spawn_local(async move {
            if api::delete_article(&token2, &slug).await.is_ok() {
                navigate(RwRoute::Home);
            }
        });
    }

    pub fn toggle_favorite(&mut self) {
        let auth = AuthStore::get();
        let token = auth.token.read();
        if token.is_empty() {
            return;
        }
        let slug = self.slug.clone();
        let article = self.article.clone();
        let token2 = token.clone();
        let slug2 = slug.clone();
        #[cfg(brick_dom)]
        spawn_local(async move {
            let favorited = if let Load::Loaded(ref a) = article.read() {
                a.favorited
            } else {
                false
            };
            let result = if favorited {
                api::unfavorite_article(&token2, &slug2).await
            } else {
                api::favorite_article(&token2, &slug2).await
            };
            if let Ok(updated) = result {
                article.set(Load::Loaded(updated));
            }
        });
    }

    pub fn toggle_follow(&mut self) {
        let auth = AuthStore::get();
        let token = auth.token.read();
        if token.is_empty() {
            return;
        }
        let article = self.article.clone();
        let token2 = token.clone();
        #[cfg(brick_dom)]
        spawn_local(async move {
            if let Load::Loaded(ref a) = article.read() {
                let username = a.author.username.clone();
                let following = a.author.following;
                let result = if following {
                    api::unfollow_user(&token2, &username).await
                } else {
                    api::follow_user(&token2, &username).await
                };
                if let Ok(profile) = result {
                    if let Load::Loaded(mut art) = article.read() {
                        art.author = profile;
                        article.set(Load::Loaded(art));
                    }
                }
            }
        });
    }
}

#[view(ArticlePage)]
fn render() -> Box<ViewComposite> {
    style! {
        .article-page { max-width: 960px; margin: 0 auto; padding: 0 1rem; }
        .article-banner { background: #333; color: white; padding: 2rem 1rem; margin-bottom: 2rem; }
        .article-banner h1 { font-size: 2rem; font-weight: 700; margin-bottom: 1rem; }
        .article-meta { display: flex; align-items: center; gap: 1rem; }
        .article-meta img { width: 32px; height: 32px; border-radius: 50%; }
        .meta-info { display: flex; flex-direction: column; }
        .meta-author { color: white; text-decoration: none; font-weight: 600; }
        .meta-date { font-size: 0.8rem; opacity: 0.7; }
        .meta-actions { display: flex; gap: 0.5rem; margin-left: 1rem; }
        .btn-outline { padding: 0.3rem 0.7rem; border: 1px solid white; background: none; color: white; cursor: pointer; border-radius: 3px; font-size: 0.85rem; }
        .article-body { padding: 1.5rem 0; border-bottom: 1px solid #ddd; margin-bottom: 1.5rem; line-height: 1.7; }
        .comment-section { max-width: 720px; margin: 0 auto 2rem; }
        .comment-form { border: 1px solid #ddd; border-radius: 4px; margin-bottom: 1.5rem; }
        .comment-form textarea { width: 100%; border: none; padding: 1rem; resize: vertical; min-height: 6rem; font-size: 0.95rem; }
        .comment-form-footer { display: flex; align-items: center; justify-content: flex-end; padding: 0.5rem 1rem; background: #f5f5f5; border-top: 1px solid #ddd; }
        .comment-card { border: 1px solid #ddd; border-radius: 4px; margin-bottom: 1rem; }
        .comment-body { padding: 1rem; }
        .comment-footer { display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem 1rem; background: #f5f5f5; border-top: 1px solid #ddd; font-size: 0.85rem; }
        .comment-author { color: #333; font-weight: 600; text-decoration: none; }
        .comment-date { color: #bbb; }
        .status-msg { color: #aaa; padding: 2rem; text-align: center; }
        .error-msg { color: #e53e3e; padding: 1rem; }
    }
    children! {
        catch!(my.article, |err| p(&format!("Error: {}", err.read())).c("error-msg"),
            wait!(my.article, p("Loading article…").c("status-msg"),
                load!(my.article, |art| {
                    let author_href = format!("/@{}", art.author.username);
                    let avatar_src = art.author.image.clone().unwrap_or_default();
                    let created = art.created_at[..10].to_string();
                    let fav_label = if art.favorited {
                        format!("♥ Unfavorite ({})", art.favorites_count)
                    } else {
                        format!("♡ Favorite ({})", art.favorites_count)
                    };
                    let follow_label = if art.author.following { "- Unfollow" } else { "+ Follow" };
                    Box::new(crate::view_components::BrickContainer {
                        class: "article-page",
                        children: vec![
                            div {
                                class("article-banner"),
                                h1(&art.title),
                                div {
                                    class("article-meta"),
                                    img().attr("src", &avatar_src).attr("alt", &art.author.username),
                                    div {
                                        class("meta-info"),
                                        a(&art.author.username).c("meta-author").attr("href", &author_href),
                                        span(&created).c("meta-date"),
                                    },
                                    div {
                                        class("meta-actions"),
                                        button(follow_label).c("btn-outline").trigger(&my.toggle_follow),
                                        button(&fav_label).c("btn-outline").trigger(&my.toggle_favorite),
                                        button("Delete Article").c("btn-outline").trigger(&my.delete_article),
                                    },
                                },
                            }.into_component(),
                            div {
                                class("article-body"),
                                p(&art.body),
                            }.into_component(),
                        ],
                    })
                }),
            ),
        ),
        div {
            class("comment-section"),
            h3("Comments"),
            when!(my.logged_in,
                div {
                    class("comment-form"),
                    input().attr("type", "text").attr("placeholder", "Write a comment…").bind(&my.new_comment),
                    div {
                        class("comment-form-footer"),
                        button("Post Comment").primary().trigger(&my.submit_comment),
                    },
                },
                p("Sign in to leave a comment."),
            ),
            catch!(my.comments, |err| p(&format!("Error loading comments: {}", err.read())).c("error-msg"),
                wait!(my.comments, p("Loading comments…").c("status-msg"),
                    load!(my.comments, |comments| {
                        let items: Vec<Box<dyn crate::view_components::Brick>> = comments.iter().map(|c| {
                            let author_href = format!("/@{}", c.author.username);
                            let avatar_src = c.author.image.clone().unwrap_or_default();
                            let created = c.created_at[..10].to_string();
                            Box::new(crate::view_components::BrickContainer {
                                class: "comment-card",
                                children: vec![
                                    p(&c.body).c("comment-body").into_component(),
                                    div {
                                        class("comment-footer"),
                                        img().attr("src", &avatar_src).attr("alt", &c.author.username),
                                        a(&c.author.username).c("comment-author").attr("href", &author_href),
                                        span(&created).c("comment-date"),
                                    }.into_component(),
                                ],
                            }) as Box<dyn crate::view_components::Brick>
                        }).collect();
                        Box::new(crate::view_components::BrickContainer {
                            class: "",
                            children: items,
                        })
                    }),
                ),
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::cascade;

    fn page() -> ArticlePage {
        ArticlePage {
            slug: "test-slug".to_string(),
            ..cascade()
        }
    }

    #[test]
    fn renders_comment_section() {
        assert!(crate::view_components::render_to_html(&*page().into_component()).contains("Comments"));
    }

    #[test]
    fn initial_article_idle() {
        assert!(matches!(page().article.read(), Load::Idle));
    }

    #[test]
    fn initial_comments_idle() {
        assert!(matches!(page().comments.read(), Load::Idle));
    }

    #[test]
    fn slug_is_preserved() {
        assert_eq!(page().slug, "test-slug");
    }
}
