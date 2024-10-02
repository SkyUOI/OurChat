use clap::Parser;
use pingora::prelude::*;
use std::sync::Arc;

pub struct LB {
    lb: Arc<LoadBalancer<RoundRobin>>,
}

impl LB {
    pub fn new(lb: Arc<LoadBalancer<RoundRobin>>) -> Self {
        Self { lb }
    }
}

#[async_trait::async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self.lb.select(b"", 256).unwrap();
        println!("upstream peer is:{:?}", upstream);
        let peer = Box::new(HttpPeer::new(upstream, true, "one.one.one.one".to_owned()));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_request
            .insert_header("Host", "one.one.one.one")
            .unwrap();
        Ok(())
    }
}

const DEFAULT_IP: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 7777;

const fn default_port() -> u16 {
    DEFAULT_PORT
}

fn default_ip() -> String {
    DEFAULT_IP.to_owned()
}

#[derive(Debug, Parser)]
#[command(author = "SkyUOI", version = base::build::VERSION, about = "The Load Balancer of OurChat")]
struct ArgParser {
    #[command(flatten)]
    pingora_arg: PingOpt,
    #[arg(short, long, default_value_t = String::from(DEFAULT_IP), help = "binding ip")]
    cfg: String,
}

// Copy from `pingora::prelude::Opt`
#[derive(Parser, Debug, Default)]
#[clap(name = "basic", long_about = None)]
pub struct PingOpt {
    /// Whether this server should try to upgrade from a running old server
    #[clap(
        short,
        long,
        help = "This is the base set of command line arguments for a pingora-based service",
        long_help = None
    )]
    pub upgrade: bool,

    /// Whether this server should run in the background
    #[clap(short, long)]
    pub daemon: bool,

    /// Not actually used. This flag is there so that the server is not upset seeing this flag
    /// passed from `cargo test` sometimes
    #[clap(long)]
    pub nocapture: bool,

    /// Test the configuration and exit
    ///
    /// When this flag is set, calling `server.bootstrap()` will exit the process without errors
    ///
    /// This flag is useful for upgrading service where the user wants to make sure the new
    /// service can start before shutting down the old server process.
    #[clap(
        short,
        long,
        help = "This flag is useful for upgrading service where the user wants \
                to make sure the new service can start before shutting down \
                the old server process.",
        long_help = None
    )]
    pub test: bool,

    /// The path to the configuration file.
    ///
    /// See [`ServerConf`] for more details of the configuration file.
    #[clap(short, long, help = "The path to the configuration file.", long_help = None)]
    pub pingora_conf: Option<String>,
}

impl From<PingOpt> for Opt {
    fn from(val: PingOpt) -> Self {
        Opt {
            upgrade: val.upgrade,
            daemon: val.daemon,
            nocapture: val.nocapture,
            test: val.test,
            conf: val.pingora_conf,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Cfg {
    #[serde(default = "default_ip")]
    ip: String,
    #[serde(default = "default_port")]
    port: u16,
    available_servers: Vec<String>,
}

fn main() {
    // parser
    let parser = ArgParser::parse();
    let opt: Opt = parser.pingora_arg.into();
    // cfg file
    let cfg = config::Config::builder()
        .add_source(config::File::with_name(&parser.cfg))
        .build()
        .expect("Failed to build config");
    let cfg: Cfg = cfg.try_deserialize().expect("wrong config file");
    let addr = format!("{}:{}", cfg.ip, cfg.port);

    let mut pingora_server = Server::new(Some(opt)).unwrap();
    pingora_server.bootstrap();
    let mut upstreams = LoadBalancer::try_from_iter(cfg.available_servers.iter()).unwrap();

    let hc = TcpHealthCheck::new();
    upstreams.set_health_check(hc);
    upstreams.health_check_frequency = Some(std::time::Duration::from_secs(2));

    let background = background_service("health check", upstreams);
    let upstreams = background.task();

    let mut lb = http_proxy_service(&pingora_server.configuration, LB::new(upstreams));
    lb.add_tcp(&addr);

    pingora_server.add_service(background);

    pingora_server.add_service(lb);
    pingora_server.run_forever();
}
