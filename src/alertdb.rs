use aws_sdk_dynamodb::model::{PutRequest, WriteRequest};
use std::collections::HashMap;
use tokio_stream::StreamExt;

use crate::bamboo::TimeOffRequest;

// AlertDB is a DynamoDB client wrapper with some extra info about the alert DB
// table
#[derive(Debug)]
pub struct AlertDB {
    pub table_name: String,
    pub dynamodb_client: aws_sdk_dynamodb::Client,
}

#[derive(Debug)]
pub struct Item;

impl AlertDB {
    pub fn new(client: aws_sdk_dynamodb::Client, table_name: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            dynamodb_client: client,
        }
    }

    pub async fn get_all_items(
        &self,
    ) -> Result<Vec<TimeOffRequest>, Box<(dyn std::error::Error + Send + Sync)>> {
        let items: Result<Vec<_>, _> = self
            .dynamodb_client
            .scan()
            .table_name(&self.table_name)
            .into_paginator()
            .items()
            .send()
            .collect()
            .await;

        // HashMap<std::string::String, AttributeValue>
        let items = items?;

        let time_off_requests: Vec<TimeOffRequest> = serde_dynamo::from_items(items)?;

        Ok(time_off_requests)
    }

    pub async fn write_one_item(
        &self,
        time_off_request: &TimeOffRequest,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let item = serde_dynamo::to_item(time_off_request)?;
        // dbg!(&item);

        let dynamodb_request = self
            .dynamodb_client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item));

        dbg!(
            "Executing request [{:?}] to add item ...",
            &dynamodb_request
        );
        dynamodb_request.send().await?;

        Ok(())
    }

    // write_new_items writes a vec of TimeOffRequest to the DynamoDB table
    pub async fn write_new_items(
        &self,
        time_off_requests: &[TimeOffRequest],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // short-circuit if there are no requests to process
        if time_off_requests.is_empty() {
            return Ok(());
        }

        // prepare a writeRequest for processing by batchProcessRecords
        let write_request_slice: Vec<WriteRequest> = time_off_requests
            .iter()
            .map(|req| serde_dynamo::to_item(req).unwrap())
            .map(|item| PutRequest::builder().set_item(Some(item)).build())
            .map(|put_req| {
                WriteRequest::builder()
                    .set_put_request(Some(put_req))
                    .build()
            })
            .collect();
        let batch_request = self.make_batch_request(&write_request_slice);

        self.batch_process_records(batch_request).await?;
        Ok(())
    }

    fn make_batch_request(
        &self,
        request_list: &[aws_sdk_dynamodb::model::WriteRequest],
    ) -> HashMap<String, Vec<aws_sdk_dynamodb::model::WriteRequest>> {
        let mut map = HashMap::new();
        map.insert(self.table_name.to_string(), request_list.to_vec());
        map
    }

    // batchProcessRecords processes a dynamodb BatchWriteItem request with proper
    // chunking and result checking / backoff
    async fn batch_process_records(
        &self,
        batch_request: HashMap<String, Vec<aws_sdk_dynamodb::model::WriteRequest>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let opt_req_val = batch_request.get(&self.table_name);
        if opt_req_val.is_none() {
            return Err(format!("request vec not found for table: {}", &self.table_name).into());
        }
        // we just checked if none above, so this unwrap is ok.
        let orig_request_vec = opt_req_val.unwrap();

        // short-circuit if there are no requests to process
        if orig_request_vec.is_empty() {
            dbg!("no requests to process... short-circuiting");
            return Ok(());
        }

        let max_chunk_size = 25;
        let chunk_size = std::cmp::min(max_chunk_size, orig_request_vec.len());
        dbg!(&chunk_size);
        let num_full_chunks = orig_request_vec.len() / chunk_size;
        dbg!(&num_full_chunks);
        let rest_chunk_size = orig_request_vec.len() % chunk_size;
        dbg!(&rest_chunk_size);

        // this makes 'chunks' an int vec like [25, 25, 25, 17], for example
        let mut chunks: Vec<usize> = vec![chunk_size; num_full_chunks];
        if rest_chunk_size > 0 {
            chunks.push(rest_chunk_size);
        }

        let mut prev = 0;
        let mut request_vec: Vec<_>;

        dbg!(&chunks);
        for chunk_len in chunks {
            let start = prev;
            let end = start + chunk_len;
            request_vec = orig_request_vec[start..end].to_vec();
            // process BatchWriteInput here...
            let batch_write_request = self
                .dynamodb_client
                .batch_write_item()
                .set_request_items(Some(self.make_batch_request(&request_vec)))
                .return_consumed_capacity(aws_sdk_dynamodb::model::ReturnConsumedCapacity::Indexes)
                .return_item_collection_metrics(
                    aws_sdk_dynamodb::model::ReturnItemCollectionMetrics::Size,
                );
            // dbg!(&batch_write_request);
            let batch_write_output = batch_write_request.send().await?;
            // dbg!(&batch_write_output);
            let mut iteration = 1;

            if let Some(unprocessed) = batch_write_output.unprocessed_items {
                let empty_wr_vec = Vec::<WriteRequest>::new();
                let mut unprocessed_write_reqs = unprocessed
                    .get(&self.table_name)
                    .unwrap_or(&empty_wr_vec)
                    .clone();
                dbg!(&unprocessed_write_reqs);
                while !unprocessed_write_reqs.is_empty() && iteration < 5 {
                    iteration += 1;
                    tokio::time::sleep(tokio::time::Duration::from_secs(iteration)).await;
                    let batch_write_request = self
                        .dynamodb_client
                        .batch_write_item()
                        .set_request_items(Some(self.make_batch_request(&unprocessed_write_reqs)))
                        .return_consumed_capacity(
                            aws_sdk_dynamodb::model::ReturnConsumedCapacity::Indexes,
                        )
                        .return_item_collection_metrics(
                            aws_sdk_dynamodb::model::ReturnItemCollectionMetrics::Size,
                        );
                    dbg!(&batch_write_request);
                    let batch_write_output = batch_write_request.send().await?;
                    dbg!(&batch_write_output);
                    if let Some(unprocessed) = batch_write_output.unprocessed_items {
                        // note: DO NOT SHADOW THIS W/`let` HERE!! it must be
                        // the same variable for the while loop to work
                        // correctly.
                        unprocessed_write_reqs = unprocessed
                            .get(&self.table_name)
                            .unwrap_or(&empty_wr_vec)
                            .clone();
                        dbg!(&unprocessed_write_reqs);
                    }
                }
            }
            // end BatchWriteItem processing, back to chunking
            prev += chunk_len;
        }

        Ok(())
    }
}
