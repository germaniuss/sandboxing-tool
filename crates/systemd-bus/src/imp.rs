use std::ops::Deref;

use async_lock::OnceCell;
use futures_lite::StreamExt;

use crate::manager::{self, JobRemovedArgs, ManagerProxy};
use crate::properties;

static CONN: OnceCell<zbus::Connection> = OnceCell::new();

async fn conn() -> zbus::Result<&'static zbus::Connection> {
    // p2p connection missing since currently is impossible to establish
    // due to zbus validation
    CONN.get_or_try_init(|| async { zbus::Connection::session().await })
        .await
}

enum Proxy<'a> {
    Borrowed(&'a ManagerProxy<'a>),
    Owned(ManagerProxy<'static>),
}

impl<'a> Deref for Proxy<'a> {
    type Target = ManagerProxy<'a>;
    fn deref(&self) -> &Self::Target {
        match self {
            Proxy::Borrowed(p) => *p,
            Proxy::Owned(p) => p,
        }
    }
}

async fn _proxy<'a>(proxy: Option<&'a ManagerProxy<'a>>) -> zbus::Result<Proxy<'a>> {
    if let Some(p) = proxy {
        Ok(Proxy::Borrowed(p))
    } else {
        let conn = conn().await?;
        let proxy = ManagerProxy::new(conn).await?;
        Ok(Proxy::Owned(proxy))
    }
}

// Functions

pub async fn start<'a, T>(
    unit: &str,
    mode: &str,
    properties: Option<T>,
    #[allow(unused_variables)] flags: u64,
    proxy: Option<&manager::ManagerProxy<'_>>,
) -> zbus::Result<()>
where
    T: properties::ToProperties<'a>,
{
    let proxy = _proxy(proxy).await?;

    let job_path = if let Some(properties_v) = properties {
        proxy
            .start_transient_unit(unit, mode, &properties_v.into(), &[])
            .await?
    } else {
        // The flags are currently not used for anything
        // so we set them to 0
        proxy.start_unit_with_flags(unit, mode, 0).await?
    };

    let mut job_result: String = "".into();
    let mut job_removed_stream = proxy.receive_job_removed().await?;
    while let Some(msg) = job_removed_stream.next().await {
        let args: JobRemovedArgs = msg.args()?;
        if args.job.as_str() == job_path.as_str() {
            job_result = args.result.into();
            break;
        }
    }

    if job_result != "done" {
        return Err(zbus::Error::Failure(format!(
            "Unit initialization failed with job result: {}",
            job_result
        )));
    }

    Ok(())
}
