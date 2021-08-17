import * as cdk from '@aws-cdk/core';
import * as lambda from '@aws-cdk/aws-lambda';
import * as s3 from '@aws-cdk/aws-s3';
import * as apigateway from '@aws-cdk/aws-apigateway';
import { DockerImage } from '@aws-cdk/aws-cloudwatch/node_modules/@aws-cdk/core';
import { RetentionDays } from '@aws-cdk/aws-logs';


export class S3FileManagerStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const bkt = new s3.Bucket(this, 'files-bucket', {
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
    });

    const target = 'x86_64-unknown-linux-musl';
    const hello = new lambda.Function(this, 'HelloHandler', {
      code: lambda.Code.fromAsset('resources/s3-file-backend', {
        bundling: {
          command: [
            'sh', '-c',
            `apk update && apk add build-base && rustup target add ${target} && cargo build --release --target ${target} && cp target/${target}/release/bootstrap /asset-output/bootstrap`
          ],
          image: DockerImage.fromRegistry('rust:alpine'),
          user: 'root'
        }
      }),
      functionName: 'hello',
      handler: 'main',
      runtime: lambda.Runtime.PROVIDED_AL2,
      environment: {
        'BUCKET': bkt.bucketName,
        'RUST_LOG': 's3-file-backend=trace'
      },
      logRetention: RetentionDays.ONE_WEEK,
    });
    bkt.grantRead(hello);

    const api = new apigateway.RestApi(this, 'S3 File sharing API gateway');

    const files = api.root.addResource('files');
    const file = files.addResource('{file}');
    file.addMethod('GET', new apigateway.LambdaIntegration(hello));

  }
}
