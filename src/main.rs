use yakyak::config::Config;
use yakyak::domain::call::{Call, CallDirection, Participant};
use yakyak::domain::shared::value_objects::{CallId, EndpointId, SessionId, SipUri};
use yakyak::infrastructure::protocols::sip::{
    AckHandler, ByeHandler, CancelHandler, InviteHandler, Registrar, SipMethod, SipServer,
    SipServerConfig,
};
use std::net::IpAddr;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

#[cfg(feature = "postgres")]
use yakyak::infrastructure::persistence::{create_pool, run_migrations, DatabaseConfig, PgUserRepository, PgCdrRepository};
#[cfg(feature = "postgres")]
use yakyak::infrastructure::protocols::sip::DigestAuthDb;
#[cfg(feature = "postgres")]
use yakyak::interface::api::{build_router, init_metrics, update_active_calls, update_registered_users, AppState, EventBroadcaster};
#[cfg(not(feature = "postgres"))]
use yakyak::infrastructure::protocols::sip::DigestAuth;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting YakYak PBX System");

    // Load configuration
    let config = Config::default();
    info!("Configuration loaded: {:?}", config);

    // Demo: Create a sample call to verify domain model
    demo_call_lifecycle().await?;

    info!("YakYak PBX System initialized successfully");

    // Initialize database and API server (if postgres feature is enabled)
    #[cfg(feature = "postgres")]
    let (user_repository, cdr_repository): (Arc<dyn yakyak::domain::user::UserRepository>, Option<Arc<dyn yakyak::domain::cdr::CdrRepository>>) = {
        info!("Initializing database connection...");

        // Create database pool
        let db_config = DatabaseConfig {
            url: config.database.url.clone(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout: std::time::Duration::from_secs(30),
            idle_timeout: std::time::Duration::from_secs(600),
            max_lifetime: std::time::Duration::from_secs(1800),
        };

        let pool = create_pool(&db_config).await?;
        info!("Database connection pool created");

        // Run migrations
        info!("Running database migrations...");
        run_migrations(&pool).await?;
        info!("Database migrations completed");

        // Create user repository
        let user_repo: Arc<dyn yakyak::domain::user::UserRepository> = Arc::new(PgUserRepository::new(pool.clone()));
        info!("User repository initialized");

        // Create CDR repository
        let cdr_repo: Arc<dyn yakyak::domain::cdr::CdrRepository> = Arc::new(PgCdrRepository::new(pool.clone()));
        info!("CDR repository initialized");

        (user_repo, Some(cdr_repo))
    };

    #[cfg(not(feature = "postgres"))]
    let user_repository: Option<Arc<dyn yakyak::domain::user::UserRepository>> = None;
    #[cfg(not(feature = "postgres"))]
    let cdr_repository: Option<Arc<dyn yakyak::domain::cdr::CdrRepository>> = None;

    // Start SIP server
    let sip_config = SipServerConfig {
        udp_bind: format!("{}:{}", config.sip.bind_address, config.sip.bind_port)
            .parse()
            .unwrap(),
        tcp_bind: format!("{}:{}", config.sip.bind_address, config.sip.bind_port)
            .parse()
            .unwrap(),
        domain: config.sip.domain.clone(),
        enable_tcp: true,
    };

    let mut sip_server = SipServer::new(sip_config);

    // Initialize authentication
    #[cfg(feature = "postgres")]
    let auth = Arc::new(DigestAuthDb::new(config.sip.domain.clone(), user_repository.clone()));

    #[cfg(not(feature = "postgres"))]
    let auth = {
        let auth = Arc::new(DigestAuth::new(&config.sip.domain));
        // Add test users (in-memory fallback when database is not available)
        auth.add_user("alice", "secret123").await;
        auth.add_user("bob", "secret456").await;
        info!("Added test users: alice, bob (in-memory)");
        auth
    };

    // Register SIP handlers with authentication
    let registrar = Arc::new(Registrar::with_auth(auth.clone()));
    sip_server
        .register_handler(SipMethod::Register, registrar.clone())
        .await;

    // Register call handlers with authentication
    let local_ip: IpAddr = "0.0.0.0".parse().unwrap(); // Use actual local IP in production

    #[cfg(feature = "postgres")]
    let invite_handler = {
        let handler = InviteHandler::with_auth(
            registrar.clone(),
            local_ip,
            auth.clone(),
        );

        // Add CDR repository if available
        if let Some(ref cdr_repo) = cdr_repository {
            Arc::new(handler.with_cdr_repository(cdr_repo.clone()))
        } else {
            Arc::new(handler)
        }
    };

    #[cfg(not(feature = "postgres"))]
    let invite_handler = Arc::new(InviteHandler::with_auth(
        registrar.clone(),
        local_ip,
        auth.clone(),
    ));

    let active_calls = invite_handler.active_calls.clone();
    let call_router = invite_handler.call_router();

    // Start metrics updater task (if postgres feature is enabled)
    #[cfg(feature = "postgres")]
    {
        let router_clone = call_router.clone();
        let registrar_clone = registrar.clone();
        tokio::spawn(async move {
            loop {
                // Update active calls gauge
                let active_count = router_clone.active_call_count().await;
                update_active_calls(active_count);

                // Update registered users gauge
                let registered_count = registrar_clone.get_registration_count().await;
                update_registered_users(registered_count);

                // Update every 5 seconds
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
        info!("Metrics updater task started");
    }

    // Start REST API server (if postgres feature is enabled)
    #[cfg(feature = "postgres")]
    let api_server_handle = {
        info!("Starting REST API server on {}:{}", config.server.host, config.server.port);

        // Initialize metrics exporter
        info!("Initializing Prometheus metrics exporter");
        let prometheus_handle = init_metrics();

        // Initialize event broadcaster
        info!("Initializing WebSocket event broadcaster");
        let event_broadcaster = Arc::new(EventBroadcaster::new());

        let api_state = AppState {
            user_repository: user_repository.clone(),
            cdr_repository: cdr_repository.clone(),
            call_router: Some(call_router.clone()),
            registrar: Some(registrar.clone()),
            event_broadcaster: Some(event_broadcaster.clone()),
        };
        let app = build_router(api_state, prometheus_handle, event_broadcaster);
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
            .await?;

        let api_handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("API server failed");
        });

        info!("REST API server started on {}:{}", config.server.host, config.server.port);
        Some(api_handle)
    };

    #[cfg(not(feature = "postgres"))]
    let api_server_handle: Option<tokio::task::JoinHandle<()>> = None;

    sip_server
        .register_handler(SipMethod::Invite, invite_handler)
        .await;

    sip_server
        .register_handler(SipMethod::Ack, Arc::new(AckHandler::new(active_calls.clone())))
        .await;

    sip_server
        .register_handler(
            SipMethod::Cancel,
            Arc::new(CancelHandler::new(active_calls.clone(), call_router.clone())),
        )
        .await;

    sip_server
        .register_handler(
            SipMethod::Bye,
            Arc::new(ByeHandler::with_router(active_calls.clone(), call_router)),
        )
        .await;

    info!("Registered handlers: REGISTER, INVITE, ACK, CANCEL, BYE");

    // Start the SIP server
    sip_server.start().await?;

    info!("SIP server started successfully");
    info!("Listening for SIP messages on UDP/TCP port {}", config.sip.bind_port);

    // Keep the server running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    // Stop SIP server
    sip_server.stop().await?;

    // Stop API server if running
    #[cfg(feature = "postgres")]
    if let Some(handle) = api_server_handle {
        handle.abort();
        info!("API server stopped");
    }

    Ok(())
}

/// Demonstrate the call lifecycle
async fn demo_call_lifecycle() -> anyhow::Result<()> {
    info!("=== Call Lifecycle Demo ===");

    // Create participants
    let caller = Participant::new(
        EndpointId::new(),
        SipUri::parse("sip:alice@example.com").map_err(|e| anyhow::anyhow!(e))?,
        Some("Alice".to_string()),
    );

    let callee = Participant::new(
        EndpointId::new(),
        SipUri::parse("sip:bob@example.com").map_err(|e| anyhow::anyhow!(e))?,
        Some("Bob".to_string()),
    );

    // Initiate call
    let mut call = Call::new(
        CallId::new(),
        caller,
        callee,
        CallDirection::Internal,
    );
    info!("Call initiated: {} -> {}", call.caller().uri(), call.callee().uri());
    info!("Call state: {:?}", call.state());

    // Ring the call
    let session_id = SessionId::new();
    call.ring(session_id)?;
    info!("Call ringing");
    info!("Call state: {:?}", call.state());

    // Answer the call
    call.answer()?;
    info!("Call answered");
    info!("Call state: {:?}", call.state());

    // Hold the call
    call.hold()?;
    info!("Call held");
    info!("Call state: {:?}", call.state());

    // Resume the call
    call.resume()?;
    info!("Call resumed");
    info!("Call state: {:?}", call.state());

    // End the call
    call.end(yakyak::domain::call::EndReason::NormalClearing)?;
    info!("Call ended");
    info!("Call state: {:?}", call.state());
    info!("Call duration: {:?}", call.duration());

    // Show generated events
    let events = call.take_events();
    info!("Generated {} domain events", events.len());

    info!("=== Call Lifecycle Demo Complete ===");

    Ok(())
}
