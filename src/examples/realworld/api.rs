use crate::browser::fetch::fetch;
use serde::{Deserialize, Serialize};

const BASE: &str = "https://api.realworld.io/api";

// ── Domain types ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize)]
pub struct Profile {
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Article {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    #[serde(rename = "tagList")]
    pub tag_list: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub favorited: bool,
    #[serde(rename = "favoritesCount")]
    pub favorites_count: u32,
    pub author: Profile,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Comment {
    pub id: u32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub body: String,
    pub author: Profile,
}

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

// ── API wrapper types ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize)]
pub struct ArticlesResponse {
    pub articles: Vec<Article>,
    #[serde(rename = "articlesCount")]
    pub articles_count: u32,
}

#[derive(Deserialize)]
struct ArticleResponse {
    article: Article,
}

#[derive(Deserialize)]
struct CommentsResponse {
    comments: Vec<Comment>,
}

#[derive(Deserialize)]
struct UserResponse {
    user: User,
}

#[derive(Deserialize)]
struct ProfileResponse {
    profile: Profile,
}

#[derive(Deserialize)]
struct TagsResponse {
    tags: Vec<String>,
}

// ── Request body types ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Serialize)]
struct UpdateUserWrapper {
    user: UpdateUserRequest,
}

#[derive(Serialize)]
pub struct CreateArticleRequest {
    pub title: String,
    pub description: String,
    pub body: String,
    #[serde(rename = "tagList")]
    pub tag_list: Vec<String>,
}

#[derive(Serialize)]
struct CreateArticleWrapper {
    article: CreateArticleRequest,
}

#[derive(Serialize)]
pub struct UpdateArticleRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
}

#[derive(Serialize)]
struct UpdateArticleWrapper {
    article: UpdateArticleRequest,
}

#[derive(Serialize)]
struct CommentBody<'a> {
    body: &'a str,
}

#[derive(Serialize)]
struct CommentWrapper<'a> {
    comment: CommentBody<'a>,
}

#[derive(Serialize)]
struct LoginUser<'a> {
    email: &'a str,
    password: &'a str,
}
#[derive(Serialize)]
struct LoginWrapper<'a> {
    user: LoginUser<'a>,
}

#[derive(Serialize)]
struct RegisterUser<'a> {
    username: &'a str,
    email: &'a str,
    password: &'a str,
}
#[derive(Serialize)]
struct RegisterWrapper<'a> {
    user: RegisterUser<'a>,
}

// ── Fetch helpers ─────────────────────────────────────────────────────────────

fn authed(url: &str, token: &str) -> crate::browser::fetch::FetchBuilder {
    fetch(url).header("Authorization", &format!("Token {}", token))
}

// ── Public API functions ──────────────────────────────────────────────────────

pub async fn get_feed(token: &str, limit: u32, offset: u32) -> Result<ArticlesResponse, String> {
    let url = format!("{}/articles/feed?limit={}&offset={}", BASE, limit, offset);
    authed(&url, token)
        .get::<ArticlesResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_articles(
    tag: Option<&str>,
    author: Option<&str>,
    favorited: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<ArticlesResponse, String> {
    let mut params = format!("limit={}&offset={}", limit, offset);
    if let Some(t) = tag {
        params.push_str(&format!("&tag={}", t));
    }
    if let Some(a) = author {
        params.push_str(&format!("&author={}", a));
    }
    if let Some(f) = favorited {
        params.push_str(&format!("&favorited={}", f));
    }
    let url = format!("{}/articles?{}", BASE, params);
    fetch(&url)
        .get::<ArticlesResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_article(slug: &str) -> Result<Article, String> {
    let url = format!("{}/articles/{}", BASE, slug);
    fetch(&url)
        .get::<ArticleResponse>()
        .await
        .map(|r| r.article)
        .map_err(|e| e.to_string())
}

pub async fn get_comments(slug: &str) -> Result<Vec<Comment>, String> {
    let url = format!("{}/articles/{}/comments", BASE, slug);
    fetch(&url)
        .get::<CommentsResponse>()
        .await
        .map(|r| r.comments)
        .map_err(|e| e.to_string())
}

pub async fn get_tags() -> Result<Vec<String>, String> {
    let url = format!("{}/tags", BASE);
    fetch(&url)
        .get::<TagsResponse>()
        .await
        .map(|r| r.tags)
        .map_err(|e| e.to_string())
}

pub async fn get_profile(username: &str, token: Option<&str>) -> Result<Profile, String> {
    let url = format!("{}/profiles/{}", BASE, username);
    let builder = if let Some(t) = token {
        authed(&url, t)
    } else {
        fetch(&url)
    };
    builder
        .get::<ProfileResponse>()
        .await
        .map(|r| r.profile)
        .map_err(|e| e.to_string())
}

pub async fn get_profile_articles(
    username: &str,
    _token: Option<&str>,
    favorited: bool,
    limit: u32,
    offset: u32,
) -> Result<ArticlesResponse, String> {
    if favorited {
        get_articles(None, None, Some(username), limit, offset).await
    } else {
        get_articles(None, Some(username), None, limit, offset).await
    }
}

pub async fn login(email: &str, password: &str) -> Result<User, String> {
    let url = format!("{}/users/login", BASE);
    let body = LoginWrapper {
        user: LoginUser { email, password },
    };
    fetch(&url)
        .post::<_, UserResponse>(&body)
        .await
        .map(|r| r.user)
        .map_err(|e| e.to_string())
}

pub async fn register(username: &str, email: &str, password: &str) -> Result<User, String> {
    let url = format!("{}/users", BASE);
    let body = RegisterWrapper {
        user: RegisterUser {
            username,
            email,
            password,
        },
    };
    fetch(&url)
        .post::<_, UserResponse>(&body)
        .await
        .map(|r| r.user)
        .map_err(|e| e.to_string())
}

pub async fn get_current_user(token: &str) -> Result<User, String> {
    let url = format!("{}/user", BASE);
    authed(&url, token)
        .get::<UserResponse>()
        .await
        .map(|r| r.user)
        .map_err(|e| e.to_string())
}

pub async fn update_user(token: &str, user: UpdateUserRequest) -> Result<User, String> {
    let url = format!("{}/user", BASE);
    let body = UpdateUserWrapper { user };
    authed(&url, token)
        .put::<_, UserResponse>(&body)
        .await
        .map(|r| r.user)
        .map_err(|e| e.to_string())
}

pub async fn create_article(token: &str, req: CreateArticleRequest) -> Result<Article, String> {
    let url = format!("{}/articles", BASE);
    let body = CreateArticleWrapper { article: req };
    authed(&url, token)
        .post::<_, ArticleResponse>(&body)
        .await
        .map(|r| r.article)
        .map_err(|e| e.to_string())
}

pub async fn update_article(
    token: &str,
    slug: &str,
    req: UpdateArticleRequest,
) -> Result<Article, String> {
    let url = format!("{}/articles/{}", BASE, slug);
    let body = UpdateArticleWrapper { article: req };
    authed(&url, token)
        .put::<_, ArticleResponse>(&body)
        .await
        .map(|r| r.article)
        .map_err(|e| e.to_string())
}

pub async fn delete_article(token: &str, slug: &str) -> Result<(), String> {
    #[derive(Deserialize)]
    struct Empty {}
    let url = format!("{}/articles/{}", BASE, slug);
    authed(&url, token)
        .delete::<Empty>()
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub async fn add_comment(token: &str, slug: &str, body: &str) -> Result<Comment, String> {
    #[derive(Deserialize)]
    struct CommentResponse {
        comment: Comment,
    }
    let url = format!("{}/articles/{}/comments", BASE, slug);
    let req = CommentWrapper {
        comment: CommentBody { body },
    };
    authed(&url, token)
        .post::<_, CommentResponse>(&req)
        .await
        .map(|r| r.comment)
        .map_err(|e| e.to_string())
}

pub async fn delete_comment(token: &str, slug: &str, id: u32) -> Result<(), String> {
    #[derive(Deserialize)]
    struct Empty {}
    let url = format!("{}/articles/{}/comments/{}", BASE, slug, id);
    authed(&url, token)
        .delete::<Empty>()
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub async fn favorite_article(token: &str, slug: &str) -> Result<Article, String> {
    let url = format!("{}/articles/{}/favorite", BASE, slug);
    authed(&url, token)
        .post::<_, ArticleResponse>(&())
        .await
        .map(|r| r.article)
        .map_err(|e| e.to_string())
}

pub async fn unfavorite_article(token: &str, slug: &str) -> Result<Article, String> {
    #[derive(Deserialize)]
    struct Wrap {
        article: Article,
    }
    let url = format!("{}/articles/{}/favorite", BASE, slug);
    authed(&url, token)
        .delete::<Wrap>()
        .await
        .map(|r| r.article)
        .map_err(|e| e.to_string())
}

pub async fn follow_user(token: &str, username: &str) -> Result<Profile, String> {
    let url = format!("{}/profiles/{}/follow", BASE, username);
    authed(&url, token)
        .post::<_, ProfileResponse>(&())
        .await
        .map(|r| r.profile)
        .map_err(|e| e.to_string())
}

pub async fn unfollow_user(token: &str, username: &str) -> Result<Profile, String> {
    let url = format!("{}/profiles/{}/follow", BASE, username);
    authed(&url, token)
        .delete::<ProfileResponse>()
        .await
        .map(|r| r.profile)
        .map_err(|e| e.to_string())
}
