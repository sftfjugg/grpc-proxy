use crate::infra::dynamic::DynClient;
use hyper::http::HeaderValue;
use hyper::HeaderMap;
use std::collections::HashMap;
use std::sync::Arc;

mod dyn_map_impl;
mod md_filters_impl;
mod proxy_sink;
mod query_analysis_impl;
mod server;

use crate::app::dyn_map_impl::DynMapDefault;
use crate::app::md_filters_impl::MetadataAnalysisDefaultImpl;
use crate::app::query_analysis_impl::QueryAnalysisDefaultImpl;
use crate::config::Config;
use crate::infra::server::{
    HyperHttpServerBuilder, LogMiddle, RequestIdMiddle, ShutDown, TimeMiddle,
};
pub use proxy_sink::*;
pub use server::AppEntity;

pub trait DynMap: Send + Sync {
    fn get(&self, path: String) -> Option<Arc<DynClient>>;
    fn set(&self, name: String, path: String, dc: DynClient);
}
pub trait QueryAnalysis: Send + Sync {
    fn analysis(&self, query: &str) -> Option<HashMap<String, String>>;
}
pub trait MetadataAnalysis: Send + Sync {
    fn request(&self, header: &HeaderMap<HeaderValue>) -> HashMap<String, String>;
    fn response(&self, header: HashMap<String, String>) -> HashMap<String, String>;
}

pub async fn start(sd: ShutDown, cfg: Config) {
    let map = Arc::new(DynMapDefault::default());
    let app = AppEntity::new(
        map.clone(),
        Arc::new(QueryAnalysisDefaultImpl),
        Arc::new(MetadataAnalysisDefaultImpl::from(&cfg.metadata_filters)),
    );
    init_proxy_sink(map.clone(), cfg.proxy_sink).await;

    init_env_sink(map, cfg.env_sink, cfg.server.name).await;
    //todo 开启新的服务动态监听grpc sink变化 gateway 模式

    let _ = HyperHttpServerBuilder::new()
        .set_addr(
            cfg.server
                .addr
                .parse()
                .expect("parse config server.addr error"),
        )
        .handle(app)
        .append_filter(TimeMiddle)
        .append_filter(RequestIdMiddle::new())
        .append_filter(LogMiddle)
        // .run().await.expect("http服务报错");
        .set_shutdown_singe(sd)
        .async_run();
}

pub async fn show(cfg: Config) {
    if cfg.proxy_sink.is_empty() {
        wd_log::log_warn_ln!("config[proxy_sink] is nil");
        return;
    }
    for i in cfg.proxy_sink.iter() {
        wd_log::log_info_ln!(
            "---------> start reflect grpc server[{}] <---------",
            i.name
        );
        let client = wd_log::res_panic!(init_dyn_client(i.name.clone(),i.addr.clone()).await;"init_proxy_sink: init {} failed,addr=({})",i.name,i.addr);
        let list = client.method_list();
        for i in list.iter() {
            wd_log::log_info_ln!("{} {} {}", i.0, i.1, i.2);
        }
    }
}
