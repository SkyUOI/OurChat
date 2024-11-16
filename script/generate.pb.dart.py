import os
os.system("protoc ./message/*.proto --dart_out=grpc:client/ourchat/lib google/protobuf/timestamp.proto google/protobuf/empty.proto")