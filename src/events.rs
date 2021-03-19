use csv::Writer;
use octocrab::models::{issues, Milestone, ProjectCard, User};
use octocrab::Page;
use reqwest::Url;
use serde::*;
type DateTime = chrono::DateTime<chrono::Utc>;

// Copied from octocrab::models::IssueEvent
// There are more events than Event enum defined
// Detailed: https://docs.github.com/en/developers/webhooks-and-events/issue-event-types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IssueEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub actor: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigner: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_requester: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_reviewer: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<Label>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<Milestone>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_card: Option<ProjectCard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>, // Used instead of Event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_url: Option<Url>,
    pub created_at: DateTime,
    pub issue: issues::Issue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Label {
    pub name: String,
    pub color: String,
}

#[derive(Serialize, Debug)]
struct EventRec {
    pub id: Option<i64>,
    pub node_id: Option<String>,
    pub url: Option<String>,
    pub actor_id: i64,
    pub assignee_id: Option<i64>,
    pub assigner_id: Option<i64>,
    pub review_requester_id: Option<i64>,
    pub requested_reviewer_id: Option<i64>,
    pub label: Option<String>,
    pub milestone_title: Option<String>,
    pub project_card_url: Option<Url>,
    pub event: Option<String>, // Used instead of Event
    pub commit_id: Option<String>,
    pub commit_url: Option<Url>,
    pub created_at: DateTime,
    pub issue_id: i64,

    pub sdc_repository: String,
}

impl From<IssueEvent> for EventRec {
    fn from(from: IssueEvent) -> Self {
        Self {
            id: from.id,
            node_id: from.node_id,
            url: from.url,
            actor_id: from.actor.id,
            event: from.event,
            assignee_id: from.assignee.map(|u| u.id),
            assigner_id: from.assigner.map(|u| u.id),
            review_requester_id: from.review_requester.map(|u| u.id),
            requested_reviewer_id: from.requested_reviewer.map(|u| u.id),
            label: from.label.map(|l| l.name),
            milestone_title: from.milestone.map(|m| m.title),
            project_card_url: from.project_card.map(|p| p.url),
            commit_id: from.commit_id,
            commit_url: from.commit_url,
            created_at: from.created_at,
            issue_id: from.issue.id,

            sdc_repository: String::default(),
        }
    }
}

pub struct IssueEventFetcher {
    owner: String,
    name: String,
    octocrab: octocrab::Octocrab,
}

#[derive(Serialize)]
struct EventHandler {
    #[serde(skip_serializing_if = "Option::is_none")]
    per_page: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
}

impl IssueEventFetcher {
    pub fn new(owner: String, name: String, octocrab: octocrab::Octocrab) -> Self {
        Self {
            owner,
            name,
            octocrab,
        }
    }

    pub fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    pub async fn run<T: std::io::Write>(&self, mut wtr: Writer<T>) -> octocrab::Result<()> {
        let handler = EventHandler {
            per_page: Some(100u8),
            page: None,
        };
        let route = format!(
            "repos/{owner}/{repo}/issues/events",
            owner = &self.owner,
            repo = &self.name,
        );

        let mut page: Page<IssueEvent> = self.octocrab.get(route, Some(&handler)).await?;

        let mut events: Vec<IssueEvent> = page.take_items();
        for event in events.drain(..) {
            let mut event: EventRec = event.into();
            event.sdc_repository = self.reponame();
            wtr.serialize(&event).expect("Serialize failed");
        }

        while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
            let mut events: Vec<IssueEvent> = newpage.take_items();
            for event in events.drain(..) {
                let mut event: EventRec = event.into();
                event.sdc_repository = self.reponame();
                wtr.serialize(&event).expect("Serialize failed");
            }
            page = newpage;
        }

        Ok(())
    }
}
