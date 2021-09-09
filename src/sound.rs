use anyhow::{anyhow, Result};
use web_sys::{AudioBufferSourceNode, AudioContext, AudioDestinationNode, AudioNode};

pub fn create_audio_context() -> Result<AudioContext> {
    AudioContext::new().map_err(|err| anyhow!("Could not create audio context: {:#?}", err))
}

fn create_buffer_source(ctx: &AudioContext) -> Result<AudioBufferSourceNode> {
    ctx.create_buffer_source()
        .map_err(|err| anyhow!("Error creating buffer source {:#?}", err))
}

fn connect_with_audio_node(
    buffer_source: &AudioBufferSourceNode,
    destination: &AudioDestinationNode,
) -> Result<AudioNode> {
    buffer_source
        .connect_with_audio_node(&destination)
        .map_err(|err| anyhow!("Error connecting audio source to destination {:#?}", err))
}
