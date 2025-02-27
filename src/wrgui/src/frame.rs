use crate::canvas::WrCanvas;
use crate::types::FrameRef;

impl FrameRef {
    pub fn renderer(&mut self) -> &mut WrCanvas {
        unsafe { self.output_data().wr_data.as_mut().unwrap() }
    }
}
