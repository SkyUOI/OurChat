// Mocks generated by Mockito 5.4.4 from annotations
// in ourchat/test/connection_test.dart.
// Do not manually edit this file.

// ignore_for_file: no_leading_underscores_for_library_prefixes
import 'dart:async' as _i4;

import 'package:async/async.dart' as _i5;
import 'package:mockito/mockito.dart' as _i1;
import 'package:stream_channel/stream_channel.dart' as _i3;
import 'package:web_socket_channel/src/channel.dart' as _i2;

// ignore_for_file: type=lint
// ignore_for_file: avoid_redundant_argument_values
// ignore_for_file: avoid_setters_without_getters
// ignore_for_file: comment_references
// ignore_for_file: deprecated_member_use
// ignore_for_file: deprecated_member_use_from_same_package
// ignore_for_file: implementation_imports
// ignore_for_file: invalid_use_of_visible_for_testing_member
// ignore_for_file: prefer_const_constructors
// ignore_for_file: unnecessary_parenthesis
// ignore_for_file: camel_case_types
// ignore_for_file: subtype_of_sealed_class

class _FakeWebSocketSink_0 extends _i1.SmartFake implements _i2.WebSocketSink {
  _FakeWebSocketSink_0(
    Object parent,
    Invocation parentInvocation,
  ) : super(
          parent,
          parentInvocation,
        );
}

class _FakeStreamChannel_1<T> extends _i1.SmartFake
    implements _i3.StreamChannel<T> {
  _FakeStreamChannel_1(
    Object parent,
    Invocation parentInvocation,
  ) : super(
          parent,
          parentInvocation,
        );
}

/// A class which mocks [WebSocketChannel].
///
/// See the documentation for Mockito's code generation for more information.
class MockWebSocketChannel extends _i1.Mock implements _i2.WebSocketChannel {
  MockWebSocketChannel() {
    _i1.throwOnMissingStub(this);
  }

  @override
  _i4.Future<void> get ready => (super.noSuchMethod(
        Invocation.getter(#ready),
        returnValue: _i4.Future<void>.value(),
      ) as _i4.Future<void>);

  @override
  _i4.Stream<dynamic> get stream => (super.noSuchMethod(
        Invocation.getter(#stream),
        returnValue: _i4.Stream<dynamic>.empty(),
      ) as _i4.Stream<dynamic>);

  @override
  _i2.WebSocketSink get sink => (super.noSuchMethod(
        Invocation.getter(#sink),
        returnValue: _FakeWebSocketSink_0(
          this,
          Invocation.getter(#sink),
        ),
      ) as _i2.WebSocketSink);

  @override
  void pipe(_i3.StreamChannel<dynamic>? other) => super.noSuchMethod(
        Invocation.method(
          #pipe,
          [other],
        ),
        returnValueForMissingStub: null,
      );

  @override
  _i3.StreamChannel<S> transform<S>(
          _i3.StreamChannelTransformer<S, dynamic>? transformer) =>
      (super.noSuchMethod(
        Invocation.method(
          #transform,
          [transformer],
        ),
        returnValue: _FakeStreamChannel_1<S>(
          this,
          Invocation.method(
            #transform,
            [transformer],
          ),
        ),
      ) as _i3.StreamChannel<S>);

  @override
  _i3.StreamChannel<dynamic> transformStream(
          _i4.StreamTransformer<dynamic, dynamic>? transformer) =>
      (super.noSuchMethod(
        Invocation.method(
          #transformStream,
          [transformer],
        ),
        returnValue: _FakeStreamChannel_1<dynamic>(
          this,
          Invocation.method(
            #transformStream,
            [transformer],
          ),
        ),
      ) as _i3.StreamChannel<dynamic>);

  @override
  _i3.StreamChannel<dynamic> transformSink(
          _i5.StreamSinkTransformer<dynamic, dynamic>? transformer) =>
      (super.noSuchMethod(
        Invocation.method(
          #transformSink,
          [transformer],
        ),
        returnValue: _FakeStreamChannel_1<dynamic>(
          this,
          Invocation.method(
            #transformSink,
            [transformer],
          ),
        ),
      ) as _i3.StreamChannel<dynamic>);

  @override
  _i3.StreamChannel<dynamic> changeStream(
          _i4.Stream<dynamic> Function(_i4.Stream<dynamic>)? change) =>
      (super.noSuchMethod(
        Invocation.method(
          #changeStream,
          [change],
        ),
        returnValue: _FakeStreamChannel_1<dynamic>(
          this,
          Invocation.method(
            #changeStream,
            [change],
          ),
        ),
      ) as _i3.StreamChannel<dynamic>);

  @override
  _i3.StreamChannel<dynamic> changeSink(
          _i4.StreamSink<dynamic> Function(_i4.StreamSink<dynamic>)? change) =>
      (super.noSuchMethod(
        Invocation.method(
          #changeSink,
          [change],
        ),
        returnValue: _FakeStreamChannel_1<dynamic>(
          this,
          Invocation.method(
            #changeSink,
            [change],
          ),
        ),
      ) as _i3.StreamChannel<dynamic>);

  @override
  _i3.StreamChannel<S> cast<S>() => (super.noSuchMethod(
        Invocation.method(
          #cast,
          [],
        ),
        returnValue: _FakeStreamChannel_1<S>(
          this,
          Invocation.method(
            #cast,
            [],
          ),
        ),
      ) as _i3.StreamChannel<S>);
}
