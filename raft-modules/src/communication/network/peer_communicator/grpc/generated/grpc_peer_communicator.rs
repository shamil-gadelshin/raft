///Request vote RPC request
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteRequest {
    ///current election term
    #[prost(uint64, tag = "1")]
    pub term: u64,
    ///candidate id for the current term
    #[prost(uint64, tag = "2")]
    pub candidate_id: u64,
    ///term of the candidate last log entry
    #[prost(uint64, tag = "3")]
    pub last_log_term: u64,
    ///index of the candidate last log entry
    #[prost(uint64, tag = "4")]
    pub last_log_index: u64,
}
///Request vote RPC response
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteResponse {
    ///current election term
    #[prost(uint64, tag = "1")]
    pub term: u64,
    ///voting result
    #[prost(bool, tag = "2")]
    pub vote_granted: bool,
    ///voting peer id
    #[prost(uint64, tag = "3")]
    pub peer_id: u64,
}
///Append Entries RPC request
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppendEntriesRequest {
    ///current term
    #[prost(uint64, tag = "1")]
    pub term: u64,
    ///term of the previous log entry
    #[prost(uint64, tag = "2")]
    pub prev_log_term: u64,
    ///index of the previous log entry
    #[prost(uint64, tag = "3")]
    pub prev_log_index: u64,
    ///cluster leader id
    #[prost(uint64, tag = "4")]
    pub leader_id: u64,
    ///leader current commit index
    #[prost(uint64, tag = "5")]
    pub leader_commit: u64,
    ///log entries (empty means - request is heartbeat)
    #[prost(message, repeated, tag = "6")]
    pub entries: ::std::vec::Vec<LogEntry>,
}
///log entry
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogEntry {
    ///entry index
    #[prost(uint64, tag = "1")]
    pub index: u64,
    ///entry term
    #[prost(uint64, tag = "2")]
    pub term: u64,
    ///entry content type
    #[prost(enumeration = "ContentType", tag = "3")]
    pub content_type: i32,
    ///data in binary format, filled if content_type = Data
    #[prost(bytes, tag = "4")]
    pub data: std::vec::Vec<u8>,
    ///new cluster configuration, filled if content_type = Configuration
    #[prost(uint64, repeated, tag = "5")]
    pub new_cluster_configuration: ::std::vec::Vec<u64>,
}
///Append Entries RPC response
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppendEntriesResponse {
    ///current term
    #[prost(uint64, tag = "1")]
    pub term: u64,
    ///append entry success flag
    #[prost(bool, tag = "3")]
    pub success: bool,
}
/// Client RPC response status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ContentType {
    ///invalid type
    Unknown = 0,
    ///data content type
    Data = 1,
    ///configuration content type
    Configuration = 2,
}
pub mod client {
    use super::{AppendEntriesRequest, AppendEntriesResponse, VoteRequest, VoteResponse};
    use ::tower_grpc::codegen::client::*;

    /// Peer request service: Append Entries RPC and Vote Request RPC
    #[derive(Debug, Clone)]
    pub struct PeerRequestHandler<T> {
        inner: grpc::Grpc<T>,
    }

    impl<T> PeerRequestHandler<T> {
        pub fn new(inner: T) -> Self {
            let inner = grpc::Grpc::new(inner);
            Self { inner }
        }

        /// Poll whether this client is ready to send another request.
        pub fn poll_ready<R>(&mut self) -> futures::Poll<(), grpc::Status>
        where
            T: grpc::GrpcService<R>,
        {
            self.inner.poll_ready()
        }

        /// Get a `Future` of when this client is ready to send another request.
        pub fn ready<R>(self) -> impl futures::Future<Item = Self, Error = grpc::Status>
        where
            T: grpc::GrpcService<R>,
        {
            futures::Future::map(self.inner.ready(), |inner| Self { inner })
        }

        /// Peer request service: Append Entries RPC and Vote Request RPC
        pub fn append_entries<R>(
            &mut self,
            request: grpc::Request<AppendEntriesRequest>,
        ) -> grpc::unary::ResponseFuture<AppendEntriesResponse, T::Future, T::ResponseBody>
        where
            T: grpc::GrpcService<R>,
            grpc::unary::Once<AppendEntriesRequest>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static(
                "/grpc_peer_communicator.PeerRequestHandler/AppendEntries",
            );
            self.inner.unary(request, path)
        }

        /// Peer request service: Append Entries RPC and Vote Request RPC
        pub fn request_vote<R>(
            &mut self,
            request: grpc::Request<VoteRequest>,
        ) -> grpc::unary::ResponseFuture<VoteResponse, T::Future, T::ResponseBody>
        where
            T: grpc::GrpcService<R>,
            grpc::unary::Once<VoteRequest>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static(
                "/grpc_peer_communicator.PeerRequestHandler/RequestVote",
            );
            self.inner.unary(request, path)
        }
    }
}

pub mod server {
    use super::{AppendEntriesRequest, AppendEntriesResponse, VoteRequest, VoteResponse};
    use ::tower_grpc::codegen::server::*;

    // Redefine the try_ready macro so that it doesn't need to be explicitly
    // imported by the user of this generated code.
    macro_rules! try_ready {
        ($e:expr) => {
            match $e {
                Ok(futures::Async::Ready(t)) => t,
                Ok(futures::Async::NotReady) => return Ok(futures::Async::NotReady),
                Err(e) => return Err(From::from(e)),
            }
        };
    }

    /// Peer request service: Append Entries RPC and Vote Request RPC
    pub trait PeerRequestHandler: Clone {
        type AppendEntriesFuture: futures::Future<
            Item = grpc::Response<AppendEntriesResponse>,
            Error = grpc::Status,
        >;
        type RequestVoteFuture: futures::Future<
            Item = grpc::Response<VoteResponse>,
            Error = grpc::Status,
        >;

        /// append entry request to the peer
        fn append_entries(
            &mut self,
            request: grpc::Request<AppendEntriesRequest>,
        ) -> Self::AppendEntriesFuture;

        /// vote request to the peer
        fn request_vote(&mut self, request: grpc::Request<VoteRequest>) -> Self::RequestVoteFuture;
    }

    #[derive(Debug, Clone)]
    pub struct PeerRequestHandlerServer<T> {
        peer_request_handler: T,
    }

    impl<T> PeerRequestHandlerServer<T>
    where
        T: PeerRequestHandler,
    {
        pub fn new(peer_request_handler: T) -> Self {
            Self {
                peer_request_handler,
            }
        }
    }

    impl<T> tower::Service<http::Request<grpc::BoxBody>> for PeerRequestHandlerServer<T>
    where
        T: PeerRequestHandler,
    {
        type Response = http::Response<peer_request_handler::ResponseBody<T>>;
        type Error = grpc::Never;
        type Future = peer_request_handler::ResponseFuture<T>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(().into())
        }

        fn call(&mut self, request: http::Request<grpc::BoxBody>) -> Self::Future {
            use self::peer_request_handler::Kind::*;

            match request.uri().path() {
                "/grpc_peer_communicator.PeerRequestHandler/AppendEntries" => {
                    let service = peer_request_handler::methods::AppendEntries(
                        self.peer_request_handler.clone(),
                    );
                    let response = grpc::unary(service, request);
                    peer_request_handler::ResponseFuture {
                        kind: AppendEntries(response),
                    }
                }
                "/grpc_peer_communicator.PeerRequestHandler/RequestVote" => {
                    let service = peer_request_handler::methods::RequestVote(
                        self.peer_request_handler.clone(),
                    );
                    let response = grpc::unary(service, request);
                    peer_request_handler::ResponseFuture {
                        kind: RequestVote(response),
                    }
                }
                _ => peer_request_handler::ResponseFuture {
                    kind: __Generated__Unimplemented(grpc::unimplemented(format!(
                        "unknown service: {:?}",
                        request.uri().path()
                    ))),
                },
            }
        }
    }

    impl<T> tower::Service<()> for PeerRequestHandlerServer<T>
    where
        T: PeerRequestHandler,
    {
        type Response = Self;
        type Error = grpc::Never;
        type Future = futures::FutureResult<Self::Response, Self::Error>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(futures::Async::Ready(()))
        }

        fn call(&mut self, _target: ()) -> Self::Future {
            futures::ok(self.clone())
        }
    }

    impl<T> tower::Service<http::Request<tower_hyper::Body>> for PeerRequestHandlerServer<T>
    where
        T: PeerRequestHandler,
    {
        type Response = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Response;
        type Error = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Error;
        type Future = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Future;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            tower::Service::<http::Request<grpc::BoxBody>>::poll_ready(self)
        }

        fn call(&mut self, request: http::Request<tower_hyper::Body>) -> Self::Future {
            let request = request.map(|b| grpc::BoxBody::map_from(b));
            tower::Service::<http::Request<grpc::BoxBody>>::call(self, request)
        }
    }

    pub mod peer_request_handler {
        use super::super::{AppendEntriesRequest, VoteRequest};
        use super::PeerRequestHandler;
        use ::tower_grpc::codegen::server::*;

        pub struct ResponseFuture<T>
        where
            T: PeerRequestHandler,
        {
            pub(super) kind: Kind<
                // AppendEntries
                grpc::unary::ResponseFuture<
                    methods::AppendEntries<T>,
                    grpc::BoxBody,
                    AppendEntriesRequest,
                >,
                // RequestVote
                grpc::unary::ResponseFuture<methods::RequestVote<T>, grpc::BoxBody, VoteRequest>,
                // A generated catch-all for unimplemented service calls
                grpc::unimplemented::ResponseFuture,
            >,
        }

        impl<T> futures::Future for ResponseFuture<T>
        where
            T: PeerRequestHandler,
        {
            type Item = http::Response<ResponseBody<T>>;
            type Error = grpc::Never;

            fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    AppendEntries(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| ResponseBody {
                            kind: AppendEntries(body),
                        });
                        Ok(response.into())
                    }
                    RequestVote(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| ResponseBody {
                            kind: RequestVote(body),
                        });
                        Ok(response.into())
                    }
                    __Generated__Unimplemented(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| ResponseBody {
                            kind: __Generated__Unimplemented(body),
                        });
                        Ok(response.into())
                    }
                }
            }
        }

        pub struct ResponseBody<T>
        where
            T: PeerRequestHandler,
        {
            pub(super) kind:
                Kind<
                    // AppendEntries
                    grpc::Encode<
                        grpc::unary::Once<
                            <methods::AppendEntries<T> as grpc::UnaryService<
                                AppendEntriesRequest,
                            >>::Response,
                        >,
                    >,
                    // RequestVote
                    grpc::Encode<
                        grpc::unary::Once<
                            <methods::RequestVote<T> as grpc::UnaryService<VoteRequest>>::Response,
                        >,
                    >,
                    // A generated catch-all for unimplemented service calls
                    (),
                >,
        }

        impl<T> tower::HttpBody for ResponseBody<T>
        where
            T: PeerRequestHandler,
        {
            type Data = <grpc::BoxBody as grpc::Body>::Data;
            type Error = grpc::Status;

            fn is_end_stream(&self) -> bool {
                use self::Kind::*;

                match self.kind {
                    AppendEntries(ref v) => v.is_end_stream(),
                    RequestVote(ref v) => v.is_end_stream(),
                    __Generated__Unimplemented(_) => true,
                }
            }

            fn poll_data(&mut self) -> futures::Poll<Option<Self::Data>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    AppendEntries(ref mut v) => v.poll_data(),
                    RequestVote(ref mut v) => v.poll_data(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }

            fn poll_trailers(&mut self) -> futures::Poll<Option<http::HeaderMap>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    AppendEntries(ref mut v) => v.poll_trailers(),
                    RequestVote(ref mut v) => v.poll_trailers(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }
        }

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub(super) enum Kind<AppendEntries, RequestVote, __Generated__Unimplemented> {
            AppendEntries(AppendEntries),
            RequestVote(RequestVote),
            __Generated__Unimplemented(__Generated__Unimplemented),
        }

        pub mod methods {
            use super::super::{
                AppendEntriesRequest, AppendEntriesResponse, PeerRequestHandler, VoteRequest,
                VoteResponse,
            };
            use ::tower_grpc::codegen::server::*;

            pub struct AppendEntries<T>(pub T);

            impl<T> tower::Service<grpc::Request<AppendEntriesRequest>> for AppendEntries<T>
            where
                T: PeerRequestHandler,
            {
                type Response = grpc::Response<AppendEntriesResponse>;
                type Error = grpc::Status;
                type Future = T::AppendEntriesFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<AppendEntriesRequest>) -> Self::Future {
                    self.0.append_entries(request)
                }
            }

            pub struct RequestVote<T>(pub T);

            impl<T> tower::Service<grpc::Request<VoteRequest>> for RequestVote<T>
            where
                T: PeerRequestHandler,
            {
                type Response = grpc::Response<VoteResponse>;
                type Error = grpc::Status;
                type Future = T::RequestVoteFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<VoteRequest>) -> Self::Future {
                    self.0.request_vote(request)
                }
            }
        }
    }
}
