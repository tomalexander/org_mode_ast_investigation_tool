use serde::Serialize;

pub fn build_owner_tree<C>(file_contents: C) -> Result<OwnerTree, Box<dyn std::error::Error>>
where
    C: AsRef<str>,
{
    Ok(OwnerTree {
        input_source: file_contents.as_ref().to_owned(),
    })
}

#[derive(Serialize)]
pub struct OwnerTree {
    input_source: String,
}
