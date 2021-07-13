use s3::{Region, output::ListObjectsV2Output};
use std::error::Error;

pub async fn get_objects(bucket: &str) -> Result<ListObjectsV2Output, Box<dyn Error>> {
    println!("Entered function get_objects");
    let conf = s3::Config::builder()
        .region(Region::new("us-east-2"))
        .build();

    let client = s3::Client::from_conf(conf);
    let resp = client.list_objects_v2().bucket(bucket).send().await?;
    Ok(resp)
    // println!("{:?}", resp);
    // for object in resp.contents.unwrap_or_default() {
    //     println!("{:?}", object.key.unwrap_or_default());
    // }
    // Ok(())
}
