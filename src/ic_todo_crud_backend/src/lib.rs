use ic_cdk::{
    export::candid::{CandidType, Deserialize},
    query, update,
};
use std::cell::RefCell;

/*
Possible statuses ->
Todo,
InProgress,
Done,
Backlog,
*/
#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct Task {
    pub title: String,
    pub description: String,
    pub status: String,
}

type TasksStore = Vec<Task>;

thread_local! {
    static TASK_STORE: RefCell<TasksStore> = RefCell::new(Vec::new());
}

#[update]
fn add_task(task: Task) -> String {
    match TASK_STORE.try_with(|task_store| {
        let mut ts = (*task_store).borrow_mut();
        ts.push(task);
        return ts.len();
    }) {
        Ok(task_id) => format!("{}", task_id),
        Err(_e) => "Error adding task".to_string(),
    }
}

#[query]
fn get_tasks() -> Option<Vec<Task>> {
    match TASK_STORE.try_with(|task_store: &RefCell<Vec<Task>>| {
        let ts = task_store.borrow();
        return ts.clone();
    }) {
        Ok(tasks) => Some(tasks),
        Err(_e) => None,
    }
}

#[update]
fn change_task_status(task_id: u128, status: String) -> Option<String> {
    match TASK_STORE.try_with(|task_store| {
        let mut ts = (*task_store).borrow_mut();
        let id: usize = task_id as usize - 1;
        if let Some(task) = ts.get_mut(id) {
            (*task).status = status;
            return Some("done.".to_string());
        } else {
            return None;
        }
    }) {
        Ok(res) => res,
        Err(_e) => None,
    }
}

#[query]
fn get_tasks_with_pagination(page_no: u128, page_size: u128) -> Option<Vec<Task>> {
    match TASK_STORE.try_with(|task_store: &RefCell<Vec<Task>>| {
        let ts = task_store.borrow();
        let start_index: usize = (page_no as usize - 1) * page_size as usize;
        let end_index = start_index + page_size as usize;
        if start_index >= ts.len() {
            return None;
        }
        let tasks_slice = &ts[start_index..end_index.min(ts.len())];
        return Some(tasks_slice.to_vec());
    }) {
        Ok(tasks) => tasks,
        Err(_e) => None,
    }
}


#[update]
fn update_todo_description(task_id: u128, description: String) -> Option<String> {
    match TASK_STORE.try_with(|task_store| {
        let mut ts = (*task_store).borrow_mut();
        let id: usize = task_id as usize - 1;
        if let Some(task) = ts.get_mut(id) {
            (*task).description = description;
            return Some("done.".to_string());
        } else {
            return None;
        }
    }) {
        Ok(res) => res,
        Err(_e) => None,
    }
}

#[update]
fn delete_todo_by_id(task_id: u128) -> Option<String> {
    match TASK_STORE.try_with(|task_store| {
        let mut ts = (*task_store).borrow_mut();
        let id: usize = task_id as usize - 1;
        if id < ts.len() {
            ts.remove(id);
            return Some("done.".to_string());
        } else {
            return None;
        }
    }) {
        Ok(res) => res,
        Err(_e) => None,
    }
}

