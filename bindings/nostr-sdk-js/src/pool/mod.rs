// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_js::error::{into_err, Result};
use nostr_js::event::JsEvent;
use nostr_js::message::JsClientMessage;
use nostr_js::types::JsFilter;
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

pub mod result;

use self::result::{JsSendEventOutput, JsSendOutput};
use crate::database::JsNostrDatabase;
use crate::duration::JsDuration;
use crate::relay::blacklist::JsRelayBlacklist;
use crate::relay::options::{
    JsNegentropyOptions, JsRelayOptions, JsRelaySendOptions, JsSubscribeOptions,
};
use crate::relay::JsRelay;

#[wasm_bindgen(js_name = RelayPool)]
pub struct JsRelayPool {
    inner: RelayPool,
}

#[wasm_bindgen(js_class = RelayPool)]
impl JsRelayPool {
    /// Create new `RelayPool` with `in-memory` database
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RelayPool::new(RelayPoolOptions::default()),
        }
    }

    /// Create new `RelayPool` with `custom` database
    #[wasm_bindgen(js_name = withDatabase)]
    pub fn with_database(database: &JsNostrDatabase) -> Self {
        let database: Arc<DynNostrDatabase> = database.into();
        Self {
            inner: RelayPool::with_database(RelayPoolOptions::default(), database),
        }
    }

    /// Start
    ///
    /// Internally call `connect` without wait for connection.
    #[wasm_bindgen]
    pub async fn start(&self) {
        self.inner.start().await
    }

    /// Stop
    ///
    /// Call `connect` or `start` to re-start relays connections
    #[wasm_bindgen]
    pub async fn stop(&self) -> Result<()> {
        self.inner.stop().await.map_err(into_err)
    }

    /// Completely shutdown pool
    #[wasm_bindgen]
    pub async fn shutdown(&self) -> Result<()> {
        self.inner.clone().shutdown().await.map_err(into_err)
    }

    /// Get database
    #[wasm_bindgen(getter)]
    pub fn database(&self) -> JsNostrDatabase {
        self.inner.database().into()
    }

    /// Get blacklist
    #[wasm_bindgen(getter)]
    pub fn blacklist(&self) -> JsRelayBlacklist {
        self.inner.blacklist().into()
    }

    // /// Get relays
    // #[wasm_bindgen]
    // pub async fn relays(&self) -> HashMap<String, Arc<Relay>> {
    //     self.inner
    //         .relays()
    //         .await
    //         .into_iter()
    //         .map(|(u, r)| (u.to_string(), Arc::new(r.into())))
    //         .collect()
    // }

    /// Get relay
    #[wasm_bindgen]
    pub async fn relay(&self, url: &str) -> Result<JsRelay> {
        Ok(self.inner.relay(url).await.map_err(into_err)?.into())
    }

    #[wasm_bindgen(js_name = addRelay)]
    pub async fn add_relay(&self, url: &str, opts: &JsRelayOptions) -> Result<bool> {
        self.inner
            .add_relay(url, opts.deref().clone())
            .await
            .map_err(into_err)
    }

    #[wasm_bindgen(js_name = removeRelay)]
    pub async fn remove_relay(&self, url: String) -> Result<()> {
        self.inner.remove_relay(url).await.map_err(into_err)
    }

    #[wasm_bindgen(js_name = removeAllRelays)]
    pub async fn remove_all_relays(&self) -> Result<()> {
        self.inner.remove_all_relays().await.map_err(into_err)
    }

    /// Connect to all added relays and keep connection alive
    #[wasm_bindgen]
    pub async fn connect(&self, connection_timeout: Option<JsDuration>) {
        self.inner.connect(connection_timeout.map(|d| *d)).await
    }

    /// Disconnect from all relays
    #[wasm_bindgen]
    pub async fn disconnect(&self) -> Result<()> {
        self.inner.disconnect().await.map_err(into_err)
    }

    /// Connect to relay
    #[wasm_bindgen(js_name = connectRelay)]
    pub async fn connect_relay(
        &self,
        url: &str,
        connection_timeout: Option<JsDuration>,
    ) -> Result<()> {
        self.inner
            .connect_relay(url, connection_timeout.map(|d| *d))
            .await
            .map_err(into_err)
    }

    // /// Get subscriptions
    // #[wasm_bindgen]
    // pub async fn subscriptions(&self) -> HashMap<String, Vec<JsFilter>> {
    //     self.inner
    //         .subscriptions()
    //         .await
    //         .into_iter()
    //         .map(|(id, filters)| {
    //             (
    //                 id.to_string(),
    //                 filters.into_iter().map(|f| Arc::new(f.into())).collect(),
    //             )
    //         })
    //         .collect()
    // }

    // /// Get filters by subscription ID
    // #[wasm_bindgen]
    // pub async fn subscription(&self, id: &str) -> Option<Vec<JsFilter>> {
    //     let id = SubscriptionId::new(id);
    //     self.inner
    //         .subscription(&id)
    //         .await
    //         .map(|f| f.into_iter().map(|f| f.into()).collect())
    // }

    /// Send client message to all connected relays
    #[wasm_bindgen(js_name = sendMsg)]
    pub async fn send_msg(
        &self,
        msg: &JsClientMessage,
        opts: &JsRelaySendOptions,
    ) -> Result<JsSendOutput> {
        Ok(self
            .inner
            .send_msg(msg.deref().clone(), **opts)
            .await
            .map_err(into_err)?
            .into())
    }

    /// Send multiple client messages at once to all connected relays
    #[wasm_bindgen(js_name = batchMsg)]
    pub async fn batch_msg(
        &self,
        msgs: Vec<JsClientMessage>,
        opts: &JsRelaySendOptions,
    ) -> Result<JsSendOutput> {
        let msgs = msgs.into_iter().map(|msg| msg.deref().clone()).collect();
        Ok(self
            .inner
            .batch_msg(msgs, **opts)
            .await
            .map_err(into_err)?
            .into())
    }

    /// Send client message to specific relays
    ///
    /// Note: **the relays must already be added!**
    #[wasm_bindgen(js_name = sendMsgTo)]
    pub async fn send_msg_to(
        &self,
        urls: Vec<String>,
        msg: &JsClientMessage,
        opts: &JsRelaySendOptions,
    ) -> Result<JsSendOutput> {
        Ok(self
            .inner
            .send_msg_to(urls, msg.deref().clone(), **opts)
            .await
            .map_err(into_err)?
            .into())
    }

    /// Send multiple client messages at once to specific relays
    ///
    /// Note: **the relays must already be added!**
    #[wasm_bindgen(js_name = batchMsgTo)]
    pub async fn batch_msg_to(
        &self,
        urls: Vec<String>,
        msgs: Vec<JsClientMessage>,
        opts: &JsRelaySendOptions,
    ) -> Result<JsSendOutput> {
        let msgs = msgs.into_iter().map(|msg| msg.deref().clone()).collect();
        Ok(self
            .inner
            .batch_msg_to(urls, msgs, **opts)
            .await
            .map_err(into_err)?
            .into())
    }

    /// Send event to **all connected relays** and wait for `OK` message
    #[wasm_bindgen(js_name = sendEvent)]
    pub async fn send_event(
        &self,
        event: &JsEvent,
        opts: &JsRelaySendOptions,
    ) -> Result<JsSendEventOutput> {
        Ok(self
            .inner
            .send_event(event.deref().clone(), **opts)
            .await
            .map_err(into_err)?
            .into())
    }

    /// Send multiple `Event` at once to **all connected relays** and wait for `OK` message
    #[wasm_bindgen(js_name = batchEvent)]
    pub async fn batch_event(
        &self,
        events: Vec<JsEvent>,
        opts: &JsRelaySendOptions,
    ) -> Result<JsSendOutput> {
        let events = events.into_iter().map(|e| e.deref().clone()).collect();
        Ok(self
            .inner
            .batch_event(events, **opts)
            .await
            .map_err(into_err)?
            .into())
    }

    /// Send event to **specific relays** and wait for `OK` message
    #[wasm_bindgen(js_name = sendEventTo)]
    pub async fn send_event_to(
        &self,
        urls: Vec<String>,
        event: &JsEvent,
        opts: &JsRelaySendOptions,
    ) -> Result<JsSendEventOutput> {
        Ok(self
            .inner
            .send_event_to(urls, event.deref().clone(), **opts)
            .await
            .map_err(into_err)?
            .into())
    }

    /// Send multiple events at once to **specific relays** and wait for `OK` message
    #[wasm_bindgen(js_name = batchEventTo)]
    pub async fn batch_event_to(
        &self,
        urls: Vec<String>,
        events: Vec<JsEvent>,
        opts: &JsRelaySendOptions,
    ) -> Result<JsSendOutput> {
        let events = events.into_iter().map(|e| e.deref().clone()).collect();
        Ok(self
            .inner
            .batch_event_to(urls, events, **opts)
            .await
            .map_err(into_err)?
            .into())
    }

    /// Subscribe to filters to all connected relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    #[wasm_bindgen]
    pub async fn subscribe(&self, filters: Vec<JsFilter>, opts: &JsSubscribeOptions) -> String {
        self.inner
            .subscribe(
                filters.into_iter().map(|f| f.deref().clone()).collect(),
                **opts,
            )
            .await
            .to_string()
    }

    /// Subscribe with custom subscription ID to all connected relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    #[wasm_bindgen(js_name = subscribeWithid)]
    pub async fn subscribe_with_id(
        &self,
        id: &str,
        filters: Vec<JsFilter>,
        opts: &JsSubscribeOptions,
    ) {
        self.inner
            .subscribe_with_id(
                SubscriptionId::new(id),
                filters.into_iter().map(|f| f.deref().clone()).collect(),
                **opts,
            )
            .await
    }

    /// Subscribe to filters to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    #[wasm_bindgen(js_name = subscribeTo)]
    pub async fn subscribe_to(
        &self,
        urls: Vec<String>,
        filters: Vec<JsFilter>,
        opts: &JsSubscribeOptions,
    ) -> Result<String> {
        let filters = filters.into_iter().map(|f| f.deref().clone()).collect();
        Ok(self
            .inner
            .subscribe_to(urls, filters, **opts)
            .await
            .map_err(into_err)?
            .to_string())
    }

    /// Subscribe to filters with custom subscription ID to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    #[wasm_bindgen(js_name = subscribeWithIdTo)]
    pub async fn subscribe_with_id_to(
        &self,
        urls: Vec<String>,
        id: String,
        filters: Vec<JsFilter>,
        opts: &JsSubscribeOptions,
    ) -> Result<()> {
        let filters = filters.into_iter().map(|f| f.deref().clone()).collect();
        self.inner
            .subscribe_with_id_to(urls, SubscriptionId::new(id), filters, **opts)
            .await
            .map_err(into_err)
    }

    /// Unsubscribe
    #[wasm_bindgen]
    pub async fn unsubscribe(&self, id: String, opts: &JsRelaySendOptions) {
        self.inner
            .unsubscribe(SubscriptionId::new(id), **opts)
            .await
    }

    /// Unsubscribe from all subscriptions
    #[wasm_bindgen(js_name = unsubscribeAll)]
    pub async fn unsubscribe_all(&self, opts: &JsRelaySendOptions) {
        self.inner.unsubscribe_all(**opts).await
    }

    // /// Get events of filters
    // ///
    // /// Get events both from **local database** and **relays**
    // #[wasm_bindgen(js_name = getEventsOf)]
    // pub async fn get_events_of(
    //     &self,
    //     filters: Vec<JsFilter>,
    //     timeout: &JsDuration,
    //     opts: &JsFilterOptions,
    // ) -> Result<Vec<JsEvent>> {
    //     let filters = filters
    //         .into_iter()
    //         .map(|f| f.deref().clone())
    //         .collect();
    //     Ok(self
    //         .inner
    //         .get_events_of(filters, **timeout, **opts)
    //         .await.map_err(into_err)?
    //         .into_iter()
    //         .map(|e| e.into())
    //         .collect())
    // }
    //
    // /// Get events of filters from **specific relays**
    // ///
    // /// Get events both from **local database** and **relays**
    // #[wasm_bindgen(js_name = getEventsFrom)]
    // pub async fn get_events_from(
    //     &self,
    //     urls: Vec<String>,
    //     filters: Vec<JsFilter>,
    //     timeout: &JsDuration,
    //     opts: &JsFilterOptions,
    // ) -> Result<Vec<JsEvent>> {
    //     let filters = filters
    //         .into_iter()
    //         .map(|f| f.deref().clone())
    //         .collect();
    //     Ok(self
    //         .inner
    //         .get_events_from(urls, filters, **timeout, **opts)
    //         .await.map_err(into_err)?
    //         .into_iter()
    //         .map(|e| e.into())
    //         .collect())
    // }

    /// Negentropy reconciliation
    ///
    /// Use events stored in database
    pub async fn reconcile(&self, filter: &JsFilter, opts: &JsNegentropyOptions) -> Result<()> {
        self.inner
            .reconcile(filter.deref().clone(), **opts)
            .await
            .map_err(into_err)
    }

    // /// Negentropy reconciliation with custom items
    // pub async fn reconcile_with_items(
    //     &self,
    //     filter: &JsFilter,
    //     items: Vec<NegentropyItem>,
    //     opts: &JsNegentropyOptions,
    // ) -> Result<()> {
    //     let items = items
    //         .into_iter()
    //         .map(|item| (**item.id, **item.timestamp))
    //         .collect();
    //     Ok(self
    //         .inner
    //         .reconcile_with_items(filter.deref().clone(), items, **opts)
    //         .await?)
    // }

    // /// Handle relay pool notifications
    // pub async fn handle_notifications(
    //     &self,
    //     handler: Arc<dyn HandleNotification>,
    // ) -> Result<()> {
    //         Ok(self.inner
    //             .handle_notifications(|notification| async {
    //                 match notification {
    //                     nostr_sdk::RelayPoolNotification::Message { relay_url, message } => {
    //                         handler
    //                             .handle_msg(relay_url.to_string(), Arc::new(message.into()))
    //                             .await;
    //                     }
    //                     nostr_sdk::RelayPoolNotification::Event {
    //                         relay_url,
    //                         subscription_id,
    //                         event,
    //                     } => {
    //                         handler
    //                             .handle(
    //                                 relay_url.to_string(),
    //                                 subscription_id.to_string(),
    //                                 Arc::new((*event).into()),
    //                             )
    //                             .await;
    //                     }
    //                     _ => (),
    //                 }
    //                 Ok(false)
    //             })
    //             .await?)
    // }
}
