use std::cell::RefCell;
use std::rc::Rc;

slint::include_modules!();

#[derive(Clone)]
struct ContactData {
    id: i32,
    name: String,
    email: String,
    phone: String,
    status: String,
}

struct Crm {
    contacts: Vec<ContactData>,
    next_id: i32,
}

impl Crm {
    fn new() -> Self {
        Self {
            contacts: vec![
                ContactData { id: 1, name: "John Doe".into(), email: "john@example.com".into(), phone: "+1 555-0101".into(), status: "Active".into() },
                ContactData { id: 2, name: "Jane Smith".into(), email: "jane@example.com".into(), phone: "+1 555-0102".into(), status: "Lead".into() },
                ContactData { id: 3, name: "Bob Wilson".into(), email: "bob@company.org".into(), phone: "+1 555-0103".into(), status: "Active".into() },
                ContactData { id: 4, name: "Alice Brown".into(), email: "alice@email.com".into(), phone: "+1 555-0104".into(), status: "Inactive".into() },
                ContactData { id: 5, name: "Charlie Davis".into(), email: "charlie@work.net".into(), phone: "+1 555-0105".into(), status: "Lead".into() },
            ],
            next_id: 6,
        }
    }

    fn add(&mut self, name: String, email: String, phone: String, status: String) {
        self.contacts.push(ContactData {
            id: self.next_id,
            name,
            email,
            phone,
            status,
        });
        self.next_id += 1;
    }

    fn update(&mut self, index: usize, name: String, email: String, phone: String, status: String) {
        if let Some(c) = self.contacts.get_mut(index) {
            c.name = name;
            c.email = email;
            c.phone = phone;
            c.status = status;
        }
    }

    fn remove(&mut self, index: usize) {
        if index < self.contacts.len() {
            self.contacts.remove(index);
        }
    }

    fn search(&self, query: &str) -> Vec<ContactData> {
        if query.is_empty() {
            return self.contacts.clone();
        }
        let q = query.to_lowercase();
        self.contacts
            .iter()
            .filter(|c| c.name.to_lowercase().contains(&q) || c.email.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }
}

fn to_model(data: &[ContactData]) -> slint::ModelRc<Contact> {
    let items: Vec<Contact> = data
        .iter()
        .map(|c| Contact {
            id: c.id,
            name: c.name.clone().into(),
            email: c.email.clone().into(),
            phone: c.phone.clone().into(),
            status: c.status.clone().into(),
        })
        .collect();
    slint::ModelRc::new(slint::VecModel::from(items))
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = App::new()?;
    let crm = Rc::new(RefCell::new(Crm::new()));

    let statuses = ["Active", "Lead", "Inactive"];

    // Initial load
    ui.set_contacts(to_model(&crm.borrow().contacts));

    // Select contact
    let ui_weak = ui.as_weak();
    let crm_clone = crm.clone();
    ui.on_select_contact(move |idx| {
        let ui = ui_weak.unwrap();
        let crm = crm_clone.borrow();
        let filtered = crm.search(&ui.get_search_text());
        if let Some(c) = filtered.get(idx as usize) {
            ui.set_form_name(c.name.clone().into());
            ui.set_form_email(c.email.clone().into());
            ui.set_form_phone(c.phone.clone().into());
            let status_idx = statuses.iter().position(|&s| s == c.status).unwrap_or(0);
            ui.set_form_status_index(status_idx as i32);
        }
    });

    // Add contact
    let ui_weak = ui.as_weak();
    let crm_clone = crm.clone();
    ui.on_add_contact(move || {
        let ui = ui_weak.unwrap();
        let name: String = ui.get_form_name().into();
        if name.is_empty() {
            return;
        }
        let email: String = ui.get_form_email().into();
        let phone: String = ui.get_form_phone().into();
        let status = statuses[ui.get_form_status_index() as usize].to_string();

        crm_clone.borrow_mut().add(name, email, phone, status);

        let filtered = crm_clone.borrow().search(&ui.get_search_text());
        ui.set_contacts(to_model(&filtered));
        ui.set_form_name("".into());
        ui.set_form_email("".into());
        ui.set_form_phone("".into());
        ui.set_form_status_index(0);
    });

    // Update contact
    let ui_weak = ui.as_weak();
    let crm_clone = crm.clone();
    ui.on_update_contact(move |idx| {
        let ui = ui_weak.unwrap();
        let search = ui.get_search_text();
        let filtered = crm_clone.borrow().search(&search);

        if let Some(c) = filtered.get(idx as usize) {
            let target_id = c.id;
            let mut crm = crm_clone.borrow_mut();
            if let Some(real_idx) = crm.contacts.iter().position(|x| x.id == target_id) {
                let name: String = ui.get_form_name().into();
                let email: String = ui.get_form_email().into();
                let phone: String = ui.get_form_phone().into();
                let status = statuses[ui.get_form_status_index() as usize].to_string();
                crm.update(real_idx, name, email, phone, status);
            }
        }

        let filtered = crm_clone.borrow().search(&search);
        ui.set_contacts(to_model(&filtered));
    });

    // Delete contact
    let ui_weak = ui.as_weak();
    let crm_clone = crm.clone();
    ui.on_delete_contact(move |idx| {
        let ui = ui_weak.unwrap();
        let search = ui.get_search_text();
        let filtered = crm_clone.borrow().search(&search);

        if let Some(c) = filtered.get(idx as usize) {
            let target_id = c.id;
            let mut crm = crm_clone.borrow_mut();
            if let Some(real_idx) = crm.contacts.iter().position(|x| x.id == target_id) {
                crm.remove(real_idx);
            }
        }

        ui.set_selected_index(-1);
        ui.set_form_name("".into());
        ui.set_form_email("".into());
        ui.set_form_phone("".into());
        ui.set_form_status_index(0);

        let filtered = crm_clone.borrow().search(&search);
        ui.set_contacts(to_model(&filtered));
    });

    // Search
    let ui_weak = ui.as_weak();
    let crm_clone = crm.clone();
    ui.on_search_changed(move |query| {
        let ui = ui_weak.unwrap();
        ui.set_selected_index(-1);
        let filtered = crm_clone.borrow().search(&query);
        ui.set_contacts(to_model(&filtered));
    });

    ui.run()
}
