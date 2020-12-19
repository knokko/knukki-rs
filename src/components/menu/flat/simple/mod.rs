use crate::*;

mod buddy;
mod domain;

pub use domain::*;
use buddy::*;

pub struct SimpleFlatMenu {

    components: Vec<ComponentEntry>,
    components_to_add: Vec<ComponentToAdd>
}

impl SimpleFlatMenu {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            components_to_add: Vec::new()
        }
    }

    pub fn add_component(&mut self, component: Box<dyn Component>, domain: ComponentDomain) {
        self.components_to_add.push(ComponentToAdd { component, domain });
    }

    fn update_internal(&mut self, own_buddy: &mut dyn ComponentBuddy) {
        while !self.components_to_add.is_empty() {
            let to_add = self.components_to_add.swap_remove(0);
            let entry_to_add = ComponentEntry {
                component: to_add.component,
                domain: to_add.domain
            };
            // TODO Create buddy and attach the component

            // For this part, it is important that subscribing twice is equivalent
            // to subscribing once.
            own_buddy.request_render();

            // Don't forget this x)
            self.components.push(entry_to_add);
        }
    }
}

impl Component for SimpleFlatMenu {
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
        self.update_internal(buddy);
    }

    #[cfg(feature = "golem_rendering")]
    fn render(&mut self, golem: &golem::Context, region: RenderRegion, buddy: &mut dyn ComponentBuddy) -> RenderResult {
        // TODO Well... render the components x)
        // TODO And compute a more accurate return value!
        RenderResult::entire()
    }
}

struct ComponentToAdd {
    component: Box<dyn Component>,
    domain: ComponentDomain
}

struct ComponentEntry {
    component: Box<dyn Component>,
    domain: ComponentDomain,
    // TODO Buddy
}

