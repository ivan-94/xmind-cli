use crate::domain::diff::{Diff, DiffEvent};
use crate::domain::path::TopicPath;
use crate::domain::topic::Topic;

pub struct MutationPlanner;

impl MutationPlanner {
    pub fn plan_add_topic(request: AddTopicRequest<'_>) -> AddTopicPlan {
        let created_path = request.parent_path.join(request.title.to_owned());
        let diff = Diff::from_events(vec![DiffEvent::Added {
            path: created_path.clone(),
        }]);

        AddTopicPlan {
            parent_id: request.parent.id.0.clone(),
            new_topic_id: request.new_topic_id.to_owned(),
            created_path,
            title: request.title.to_owned(),
            diff,
        }
    }
}

pub struct AddTopicRequest<'a> {
    pub parent: &'a Topic,
    pub parent_path: &'a TopicPath,
    pub title: &'a str,
    pub new_topic_id: &'a str,
}

pub struct AddTopicPlan {
    pub parent_id: String,
    pub new_topic_id: String,
    pub created_path: TopicPath,
    pub title: String,
    pub diff: Diff,
}

#[cfg(test)]
mod tests {
    use crate::domain::diff::DiffEvent;
    use crate::domain::path::TopicPath;
    use crate::domain::topic::{Topic, TopicId};

    use super::{AddTopicRequest, MutationPlanner};

    #[test]
    fn plans_add_topic_under_resolved_parent() {
        let parent = Topic {
            id: TopicId("topic-q2".to_owned()),
            title: "Q2".to_owned(),
            note: None,
            labels: Vec::new(),
            markers: Vec::new(),
            hyperlink: None,
            image: None,
            children: Vec::new(),
        };
        let parent_path = TopicPath::parse_selector_value("/Q2").expect("path parses");

        let plan = MutationPlanner::plan_add_topic(AddTopicRequest {
            parent: &parent,
            parent_path: &parent_path,
            title: "Refund",
            new_topic_id: "topic-refund",
        });

        assert_eq!(plan.parent_id, "topic-q2");
        assert_eq!(plan.new_topic_id, "topic-refund");
        assert_eq!(plan.created_path.to_selector_value(), "/Q2/Refund");
        assert_eq!(
            plan.diff.events(),
            &[DiffEvent::Added {
                path: TopicPath::parse_selector_value("/Q2/Refund").expect("path parses")
            }]
        );
    }
}
