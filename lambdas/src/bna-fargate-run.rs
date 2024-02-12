use aws_config::BehaviorVersion;
use aws_sdk_ecs::types::{
    AssignPublicIp, AwsVpcConfiguration, ContainerOverride, KeyValuePair, NetworkConfiguration,
    TaskOverride,
};
use bnacore::aws::{get_aws_parameter, get_aws_secrets_value};
use bnalambdas::{
    authenticate_service_account, update_pipeline, AnalysisParameters, BrokenspokePipeline,
    BrokenspokeState, Context,
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tracing::info;
use url::Url;

#[derive(Deserialize)]
struct TaskInput {
    analysis_parameters: AnalysisParameters,
    setup: Setup,
    context: Context,
}

#[derive(Deserialize)]
pub struct Setup {
    neon: Neon,
}

#[derive(Deserialize)]
struct Neon {
    host: String,
}

#[derive(Serialize)]
struct TaskOutput {
    ecs_cluster_arn: String,
    task_arn: String,
    last_status: String,
    context: Context,
}

const FARGATE_MAX_TASK: i32 = 1;

async fn function_handler(event: LambdaEvent<TaskInput>) -> Result<TaskOutput, Error> {
    // Retrieve API URL.
    let url = "https://api.peopleforbikes.xyz/bnas/analysis";

    // Authenticate the service account.
    let auth = authenticate_service_account()
        .await
        .map_err(|e| format!("cannot authenticate service account: {e}"))?;

    // Read the task inputs.
    let analysis_parameters = &event.payload.analysis_parameters;
    let neon_host = &event.payload.setup.neon.host;
    let state_machine_context = &event.payload.context;
    let (state_machine_id, _) = state_machine_context.execution.ids()?;

    // Update the pipeline status.
    let pipeline = BrokenspokePipeline {
        state_machine_id,
        state: Some(BrokenspokeState::Pipeline),
        ..Default::default()
    };
    update_pipeline(url, &auth, &pipeline)?;

    // Prepare the AWS client.
    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let ecs_client = aws_sdk_ecs::Client::new(&aws_config);

    // Retrieve secrets and parameters.
    let main_db_url = get_aws_secrets_value("DATABASE_URL", "DATABASE_URL").await?;
    let ecs_cluster_arn = get_aws_parameter("BNA_CLUSTER_ARN").await?;
    let vpc_subnets = get_aws_parameter("PRIVATE_SUBNETS").await?;
    let vpc_security_groups = get_aws_parameter("BNA_TASK_SECURITY_GROUP").await?;
    let task_definition = get_aws_parameter("BNA_TASK_DEFINITION").await?;
    let s3_bucket = get_aws_parameter("BNA_BUCKET").await?;

    // Replace the main database host with the compute endpoint.
    let mut database_url = Url::parse(&main_db_url)?;
    database_url.set_host(Some(neon_host))?;

    // Prepare the command.
    let mut container_command: Vec<String> = vec![
        "-vv".to_string(),
        "run".to_string(),
        "--with-export".to_string(),
        "s3".to_string(),
        "--s3-bucket".to_string(),
        s3_bucket,
        analysis_parameters.country.clone(),
        analysis_parameters.city.clone(),
    ];
    if analysis_parameters.region.is_some() {
        container_command.push(analysis_parameters.region.clone().unwrap());
        container_command.push(analysis_parameters.fips_code.clone().unwrap());
    };

    // Prepare and run the task.
    let container_name = "brokenspoke-analyzer".to_string();
    let container_overrides = ContainerOverride::builder()
        .name(container_name)
        .set_command(Some(container_command))
        .environment(
            KeyValuePair::builder()
                .name("DATABASE_URL".to_string())
                .value(database_url)
                .build(),
        )
        .build();
    let task_overrides = TaskOverride::builder()
        .container_overrides(container_overrides)
        .build();
    let aws_vpc_configuration = AwsVpcConfiguration::builder()
        .subnets(vpc_subnets)
        .security_groups(vpc_security_groups)
        .assign_public_ip(AssignPublicIp::Enabled)
        .build()?;
    let network_configuration = NetworkConfiguration::builder()
        .awsvpc_configuration(aws_vpc_configuration)
        .build();
    let run_task_output = ecs_client
        .run_task()
        .cluster(ecs_cluster_arn)
        .count(FARGATE_MAX_TASK)
        .launch_type(aws_sdk_ecs::types::LaunchType::Fargate)
        .network_configuration(network_configuration)
        .overrides(task_overrides)
        .task_definition(task_definition)
        .send()
        .await?;

    // Prepare the output.
    let task = run_task_output.tasks().first().unwrap();
    let output = TaskOutput {
        ecs_cluster_arn: task.cluster_arn().unwrap().into(),
        task_arn: task.task_arn().unwrap().into(),
        last_status: task.last_status().unwrap().into(),
        context: state_machine_context.clone(),
    };

    // Update the pipeline status.
    let pipeline = BrokenspokePipeline {
        state_machine_id,
        // fargate_task_id: Some(task.),
        ..Default::default()
    };
    update_pipeline(url, &auth, &pipeline)?;

    Ok(output)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await.map_err(|e| {
        info!("{e}");
        e
    })
}
