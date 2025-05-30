use std::env;
use starknet::{
    accounts::{Account, ExecutionEncoding, SingleOwnerAccount},
    core::{
        chain_id,
        types::{Call, Felt, FunctionCall, BlockId, BlockTag},
        utils::{get_selector_from_name, cairo_short_string_to_felt},
    },
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider, Url,
    },
    signers::{LocalWallet, SigningKey},
};
use starknet_ff::FieldElement;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use anyhow::{anyhow, Result};
use ethers::{prelude::account, utils::hex::ToHex};
use sha3::{Keccak256, Digest};
use starknet::*;
use starknet::accounts::ExecutionV3;
use starknet::macros::felt;
use tracing::{info, debug, error, warn};
use once_cell::sync::Lazy;

// 全局 provider
static PROVIDER: Lazy<Arc<JsonRpcClient<HttpTransport>>> = Lazy::new(|| {
    let rpc_url = env::var("STARKNET_RPC_URL").expect("必须设置 STARKNET_RPC_URL 环境变量");
    let provider = JsonRpcClient::new(HttpTransport::new(
        Url::parse(&rpc_url).expect("RPC URL 格式无效")
    ));
    Arc::new(provider)
});

// 结构体定义，对应合约中的结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDetails {
    pub owner: FieldElement,
    pub wallet_address: FieldElement,
    pub status: FieldElement,
    pub created_at: u64,
    pub last_updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyDetails {
    pub name: FieldElement,
    pub repository_url: FieldElement,
    pub license: FieldElement,
    pub metadata_json: FieldElement,
    pub status: FieldElement,
    pub created_at: u64,
    pub last_updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDetails {
    pub step_type: FieldElement,
    pub tx_hash: FieldElement,
    pub related_entity_id: FieldElement,
    pub timestamp: u64,
    pub prev_step_index: FieldElement,
}

// Allocation 相关结构定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationDetails {
    pub workflow_id: FieldElement,
    pub sign_id: FieldElement,
    pub recipient: FieldElement,
    pub amount: FieldElement,
    pub token_address: FieldElement,
    pub tx_hash: FieldElement,
    pub created_at: u64,
    pub status: FieldElement,  // 0: pending, 1: executed, 2: failed
}

// Inquire 相关结构定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InquireDetails {
    pub workflow_id: FieldElement,
    pub inquirer: FieldElement,
    pub inquiree: FieldElement,
    pub question: FieldElement,
    pub response: FieldElement,
    pub status: FieldElement,  // 0: pending, 1: responded, 2: rejected
    pub created_at: u64,
    pub responded_at: u64,
}

// Receipt 相关结构定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptDetails {
    pub workflow_id: FieldElement,
    pub dependency_url: FieldElement,
    pub tx_hash: FieldElement,
    pub created_at: u64,
    pub metadata_hash: FieldElement,
    pub metadata_uri: FieldElement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptMetadata {
    pub name: FieldElement,
    pub version: FieldElement,
    pub author: FieldElement,
    pub license: FieldElement,
}

// Sign 相关结构定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignDetails {
    pub workflow_id: FieldElement,
    pub inquire_id: FieldElement,
    pub signer: FieldElement,
    pub signature_hash: FieldElement,
    pub tx_hash: FieldElement,
    pub created_at: u64,
}

/// 添加获取账户的公共方法
pub async fn get_account() -> Result<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>, anyhow::Error> {
    // 从环境变量获取私钥
    let private_key = env::var("STARKNET_PRIVATE_KEY").expect("必须设置 STARKNET_PRIVATE_KEY 环境变量");
    info!("私钥已从环境变量读取");
    
    // 设置钱包
    let key_pair = SigningKey::from_secret_scalar(
        Felt::from_hex(&private_key).expect("无效的私钥"),
    );
    let signer = LocalWallet::from_signing_key(key_pair);

    // 从环境变量获取账户地址
    let account_address_str = env::var("STARKNET_ACCOUNT_ADDRESS").expect("必须设置 STARKNET_ACCOUNT_ADDRESS 环境变量");
    info!("账户地址已从环境变量读取");
    
    let account_address = Felt::from_hex(&account_address_str).expect("无效的账户地址");
    let chain_id = chain_id::SEPOLIA; // 使用预定义的 Sepolia 链 ID

    // 创建账户对象
    let account = SingleOwnerAccount::new(
        PROVIDER.as_ref().clone(),
        signer,
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );
    
    Ok(account)
}

/// 添加合约调用的公共方法
pub async fn call_contract_function(
    contract_address: Felt,
    selector: Felt,
    calldata: Vec<Felt>,
) -> Result<Vec<Felt>, anyhow::Error> {
    // 创建函数调用对象
    let function_call = FunctionCall {
        contract_address,
        entry_point_selector: selector,
        calldata,
    };

    info!("尝试调用合约（只读操作）...");
    
    match PROVIDER.as_ref().call(function_call, BlockId::Tag(BlockTag::Latest)).await {
        Ok(result) => {
            info!("调用成功！结果: {:?}", result);
            Ok(result)
        },
        Err(e) => {
            error!("调用失败: {:?}", e);
            error!("这可能表示参数格式不正确或函数不存在，请检查后再尝试发送交易");
            Err(anyhow!("合约调用失败: {:?}", e))
        }
    }
}

/// 创建工作流
pub async fn create_workflow(
    github_owner_str: &str,
    wallet_address_str: &str,
) -> Result<(), anyhow::Error> {
    info!("开始创建工作流，github_owner: {}, wallet_address: {}", github_owner_str, wallet_address_str);
    
    // 获取账户
    let account = get_account().await?;
    
    // 从环境变量获取合约地址
    let contract_address_str = env::var("WORKFLOW_CONTRACT_ADDRESS").expect("必须设置 WORKFLOW_CONTRACT_ADDRESS 环境变量");
    info!("合约地址已从环境变量读取: {}", contract_address_str);
    
    let contract_address = Felt::from_hex(&contract_address_str).expect("无效的合约地址");

    // 将字符串转换为 felt，确保正确编码
    let github_owner = cairo_short_string_to_felt(github_owner_str).expect("无效的GitHub用户名");
    info!("转换后的github_owner: {:?}", github_owner);
    
    // 处理钱包地址参数
    let wallet_address = Felt::from_hex(wallet_address_str).expect("无效的钱包地址");
    info!("钱包地址: {:?}", wallet_address);
    
    // 使用正确的函数选择器
    let function_selector = Felt::from_hex(
        "0x5911913ce5ab907c3a2d99993ea1a79752241ca82352c7962c5c228d183b6e"
    ).expect("无效的选择器");
    
    // 准备调用参数
    let calldata = vec![github_owner, wallet_address];
    
    // 先尝试只读调用验证函数
    let _result = match call_contract_function(contract_address, function_selector, calldata.clone()).await {
        Ok(result) => result,
        Err(e) => {
            info!("只读调用验证失败，中止交易");
            return Err(e);
        }
    };
    
    // 创建函数调用对象
    let calls = vec![Call {
        to: contract_address,
        selector: function_selector,
        calldata,
    }];

    // 执行交易
    info!("正在发送create_workflow交易...");
    let tx_result = account.execute_v3(calls).send().await?;
    info!("交易已发送！交易哈希: 0x{:x}", tx_result.transaction_hash);

    // 打印 Starkscan 链接
    info!("交易已提交到网络。请在Starkscan上查看交易状态：");
    info!("https://sepolia.starkscan.co/tx/0x{:x}", tx_result.transaction_hash);
    
    Ok(())
}

