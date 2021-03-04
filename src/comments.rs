use csv::Writer;
use octocrab::models::issues::*;
use octocrab::params;
use reqwest::Url;
use serde::*;
type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Serialize, Debug)]
pub struct CommentRec {
    pub id: u64,
    pub node_id: String,
    pub url: Url,
    pub html_url: Url,
    pub body: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub user_id: i64,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}

impl From<Comment> for CommentRec {
    fn from(from: Comment) -> CommentRec {
        CommentRec {
            id: from.id,
            node_id: from.node_id,
            url: from.url,
            html_url: from.html_url,
            body: from.body,
            body_text: from.body_text,
            body_html: from.body_html,
            user_id: from.user.id,
            created_at: from.created_at,
            updated_at: from.updated_at,
        }
    }
}

pub struct CommentFetcher {
    owner: String,
    name: String,
    octocrab: octocrab::Octocrab,
}

impl CommentFetcher {
    pub fn new(owner: String, name: String, octocrab: octocrab::Octocrab) -> Self {
        Self {
            owner,
            name,
            octocrab,
        }
    }

    pub async fn run<T: std::io::Write>(&self, mut wtr: Writer<T>) -> octocrab::Result<()> {
        let mut page = self
            .octocrab
            .issues(&self.owner, &self.name)
            .list_issue_comments()
            .per_page(100)
            .send()
            .await?;

        let mut comments: Vec<Comment> = page.take_items();
        for comment in comments.drain(..) {
            let comment: CommentRec = comment.into();
            wtr.serialize(&comment).expect("Serialize failed");
        }
        while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
            comments.extend(newpage.take_items());
            for comment in comments.drain(..) {
                let comment: CommentRec = comment.into();
                wtr.serialize(&comment).expect("Serialize failed");
            }
            page = newpage;
        }

        Ok(())
    }
}