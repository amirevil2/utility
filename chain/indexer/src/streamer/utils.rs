use actix::Addr;

use node_runtime::config::tx_cost;
use unc_indexer_primitives::IndexerTransactionWithOutcome;
use unc_parameters::RuntimeConfig;
use unc_primitives::views;

use super::errors::FailedToFetchData;
use super::fetchers::fetch_block;

pub(crate) async fn convert_transactions_sir_into_local_receipts(
    client: &Addr<unc_client::ViewClientActor>,
    runtime_config: &RuntimeConfig,
    txs: Vec<&IndexerTransactionWithOutcome>,
    block: &views::BlockView,
) -> Result<Vec<views::ReceiptView>, FailedToFetchData> {
    if txs.is_empty() {
        return Ok(vec![]);
    }
    let prev_block = fetch_block(&client, block.header.prev_hash).await?;
    let prev_block_gas_price = prev_block.header.gas_price;

    let local_receipts: Vec<views::ReceiptView> =
        txs.into_iter()
            .map(|tx| {
                let cost = tx_cost(
                    &runtime_config,
                    &unc_primitives::transaction::Transaction {
                        signer_id: tx.transaction.signer_id.clone(),
                        public_key: tx.transaction.public_key.clone(),
                        nonce: tx.transaction.nonce,
                        receiver_id: tx.transaction.receiver_id.clone(),
                        block_hash: block.header.hash,
                        actions: tx
                            .transaction
                            .actions
                            .clone()
                            .into_iter()
                            .map(|action| {
                                unc_primitives::transaction::Action::try_from(action).unwrap()
                            })
                            .collect(),
                    },
                    prev_block_gas_price,
                    true,
                );
                views::ReceiptView {
                    predecessor_id: tx.transaction.signer_id.clone(),
                    receiver_id: tx.transaction.receiver_id.clone(),
                    receipt_id: *tx.outcome.execution_outcome.outcome.receipt_ids.first().expect(
                        "The transaction ExecutionOutcome should have one receipt id in vec",
                    ),
                    receipt: views::ReceiptEnumView::Action {
                        signer_id: tx.transaction.signer_id.clone(),
                        signer_public_key: tx.transaction.public_key.clone(),
                        gas_price: cost
                            .expect("TransactionCost returned IntegerOverflowError")
                            .receipt_gas_price,
                        output_data_receivers: vec![],
                        input_data_ids: vec![],
                        actions: tx.transaction.actions.clone(),
                    },
                }
            })
            .collect();

    Ok(local_receipts)
}
