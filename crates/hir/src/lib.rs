use amt::Namespace;

pub mod amt;
pub mod arena;

pub trait ModuleResolver {
    fn resolve_path(&self) -> &Namespace;
}
