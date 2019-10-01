///apply new data entry request
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewDataRequest {
    /// data in binary format
    #[prost(bytes, tag="1")]
    pub data: std::vec::Vec<u8>,
}
///add new server to the cluster request
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddServerRequest {
    ///new server id
    #[prost(uint64, tag="1")]
    pub new_server: u64,
}
///Common Client RPC response
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientRpcResponse {
    ///response status
    #[prost(enumeration="ClientResponseStatus", tag="1")]
    pub status: i32,
    ///current leader id if any, 0 (zero) means no known leader yet
    #[prost(uint64, tag="2")]
    pub current_leader: u64,
    ///error message if ClientResponseStatus::Error
    #[prost(string, tag="3")]
    pub message: std::string::String,
}
/// Client RPC response status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ClientResponseStatus {
    ///invalid status
    Unknown = 0,
    ///success
    Ok = 1,
    ///node is follower or candidate
    NotLeader = 2,
    ///request failed: leader failed to acquire the quorum
    NoQuorum = 3,
    ///other error
    Error = 4,
}
pub mod client {
    use ::tower_grpc::codegen::client::*;
    use super::{AddServerRequest, ClientRpcResponse, NewDataRequest};

    /// Client RPC
    #[derive(Debug, Clone)]
    pub struct ClientRequestHandler<T> {
        inner: grpc::Grpc<T>,
    }

    impl<T> ClientRequestHandler<T> {
        pub fn new(inner: T) -> Self {
            let inner = grpc::Grpc::new(inner);
            Self { inner }
        }

        /// Poll whether this client is ready to send another request.
        pub fn poll_ready<R>(&mut self) -> futures::Poll<(), grpc::Status>
        where T: grpc::GrpcService<R>,
        {
            self.inner.poll_ready()
        }

        /// Get a `Future` of when this client is ready to send another request.
        pub fn ready<R>(self) -> impl futures::Future<Item = Self, Error = grpc::Status>
        where T: grpc::GrpcService<R>,
        {
            futures::Future::map(self.inner.ready(), |inner| Self { inner })
        }

        /// Client RPC
        pub fn add_server<R>(&mut self, request: grpc::Request<AddServerRequest>) -> grpc::unary::ResponseFuture<ClientRpcResponse, T::Future, T::ResponseBody>
        where T: grpc::GrpcService<R>,
              grpc::unary::Once<AddServerRequest>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static("/grpc_client_communicator.ClientRequestHandler/AddServer");
            self.inner.unary(request, path)
        }

        /// Client RPC
        pub fn new_data<R>(&mut self, request: grpc::Request<NewDataRequest>) -> grpc::unary::ResponseFuture<ClientRpcResponse, T::Future, T::ResponseBody>
        where T: grpc::GrpcService<R>,
              grpc::unary::Once<NewDataRequest>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static("/grpc_client_communicator.ClientRequestHandler/NewData");
            self.inner.unary(request, path)
        }
    }
}

pub mod server {
    use ::tower_grpc::codegen::server::*;
    use super::{AddServerRequest, ClientRpcResponse, NewDataRequest};

    // Redefine the try_ready macro so that it doesn't need to be explicitly
    // imported by the user of this generated code.
    macro_rules! try_ready {
        ($e:expr) => (match $e {
            Ok(futures::Async::Ready(t)) => t,
            Ok(futures::Async::NotReady) => return Ok(futures::Async::NotReady),
            Err(e) => return Err(From::from(e)),
        })
    }

    /// Client RPC
    pub trait ClientRequestHandler: Clone {
        type AddServerFuture: futures::Future<Item = grpc::Response<ClientRpcResponse>, Error = grpc::Status>;
        type NewDataFuture: futures::Future<Item = grpc::Response<ClientRpcResponse>, Error = grpc::Status>;

        /// add new server to the cluster
        fn add_server(&mut self, request: grpc::Request<AddServerRequest>) -> Self::AddServerFuture;

        /// add new data
        fn new_data(&mut self, request: grpc::Request<NewDataRequest>) -> Self::NewDataFuture;
    }

    #[derive(Debug, Clone)]
    pub struct ClientRequestHandlerServer<T> {
        client_request_handler: T,
    }

    impl<T> ClientRequestHandlerServer<T>
    where T: ClientRequestHandler,
    {
        pub fn new(client_request_handler: T) -> Self {
            Self { client_request_handler }
        }
    }

    impl<T> tower::Service<http::Request<grpc::BoxBody>> for ClientRequestHandlerServer<T>
    where T: ClientRequestHandler,
    {
        type Response = http::Response<client_request_handler::ResponseBody<T>>;
        type Error = grpc::Never;
        type Future = client_request_handler::ResponseFuture<T>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(().into())
        }

        fn call(&mut self, request: http::Request<grpc::BoxBody>) -> Self::Future {
            use self::client_request_handler::Kind::*;

            match request.uri().path() {
                "/grpc_client_communicator.ClientRequestHandler/AddServer" => {
                    let service = client_request_handler::methods::AddServer(self.client_request_handler.clone());
                    let response = grpc::unary(service, request);
                    client_request_handler::ResponseFuture { kind: AddServer(response) }
                }
                "/grpc_client_communicator.ClientRequestHandler/NewData" => {
                    let service = client_request_handler::methods::NewData(self.client_request_handler.clone());
                    let response = grpc::unary(service, request);
                    client_request_handler::ResponseFuture { kind: NewData(response) }
                }
                _ => {
                    client_request_handler::ResponseFuture { kind: __Generated__Unimplemented(grpc::unimplemented(format!("unknown service: {:?}", request.uri().path()))) }
                }
            }
        }
    }

    impl<T> tower::Service<()> for ClientRequestHandlerServer<T>
    where T: ClientRequestHandler,
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

    impl<T> tower::Service<http::Request<tower_hyper::Body>> for ClientRequestHandlerServer<T>
    where T: ClientRequestHandler,
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

    pub mod client_request_handler {
        use ::tower_grpc::codegen::server::*;
        use super::ClientRequestHandler;
        use super::super::{AddServerRequest, NewDataRequest};

        pub struct ResponseFuture<T>
        where T: ClientRequestHandler,
        {
            pub(super) kind: Kind<
                // AddServer
                grpc::unary::ResponseFuture<methods::AddServer<T>, grpc::BoxBody, AddServerRequest>,
                // NewData
                grpc::unary::ResponseFuture<methods::NewData<T>, grpc::BoxBody, NewDataRequest>,
                // A generated catch-all for unimplemented service calls
                grpc::unimplemented::ResponseFuture,
            >,
        }

        impl<T> futures::Future for ResponseFuture<T>
        where T: ClientRequestHandler,
        {
            type Item = http::Response<ResponseBody<T>>;
            type Error = grpc::Never;

            fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    AddServer(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: AddServer(body) }
                        });
                        Ok(response.into())
                    }
                    NewData(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: NewData(body) }
                        });
                        Ok(response.into())
                    }
                    __Generated__Unimplemented(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: __Generated__Unimplemented(body) }
                        });
                        Ok(response.into())
                    }
                }
            }
        }

        pub struct ResponseBody<T>
        where T: ClientRequestHandler,
        {
            pub(super) kind: Kind<
                // AddServer
                grpc::Encode<grpc::unary::Once<<methods::AddServer<T> as grpc::UnaryService<AddServerRequest>>::Response>>,
                // NewData
                grpc::Encode<grpc::unary::Once<<methods::NewData<T> as grpc::UnaryService<NewDataRequest>>::Response>>,
                // A generated catch-all for unimplemented service calls
                (),
            >,
        }

        impl<T> tower::HttpBody for ResponseBody<T>
        where T: ClientRequestHandler,
        {
            type Data = <grpc::BoxBody as grpc::Body>::Data;
            type Error = grpc::Status;

            fn is_end_stream(&self) -> bool {
                use self::Kind::*;

                match self.kind {
                    AddServer(ref v) => v.is_end_stream(),
                    NewData(ref v) => v.is_end_stream(),
                    __Generated__Unimplemented(_) => true,
                }
            }

            fn poll_data(&mut self) -> futures::Poll<Option<Self::Data>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    AddServer(ref mut v) => v.poll_data(),
                    NewData(ref mut v) => v.poll_data(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }

            fn poll_trailers(&mut self) -> futures::Poll<Option<http::HeaderMap>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    AddServer(ref mut v) => v.poll_trailers(),
                    NewData(ref mut v) => v.poll_trailers(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }
        }

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub(super) enum Kind<AddServer, NewData, __Generated__Unimplemented> {
            AddServer(AddServer),
            NewData(NewData),
            __Generated__Unimplemented(__Generated__Unimplemented),
        }

        pub mod methods {
            use ::tower_grpc::codegen::server::*;
            use super::super::{ClientRequestHandler, AddServerRequest, ClientRpcResponse, NewDataRequest};

            pub struct AddServer<T>(pub T);

            impl<T> tower::Service<grpc::Request<AddServerRequest>> for AddServer<T>
            where T: ClientRequestHandler,
            {
                type Response = grpc::Response<ClientRpcResponse>;
                type Error = grpc::Status;
                type Future = T::AddServerFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<AddServerRequest>) -> Self::Future {
                    self.0.add_server(request)
                }
            }

            pub struct NewData<T>(pub T);

            impl<T> tower::Service<grpc::Request<NewDataRequest>> for NewData<T>
            where T: ClientRequestHandler,
            {
                type Response = grpc::Response<ClientRpcResponse>;
                type Error = grpc::Status;
                type Future = T::NewDataFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<NewDataRequest>) -> Self::Future {
                    self.0.new_data(request)
                }
            }
        }
    }
}
