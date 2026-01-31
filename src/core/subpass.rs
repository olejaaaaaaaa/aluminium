use ash::vk;

pub struct Subpass {
    pub(crate) raw: vk::SubpassDescription<'static>,
    _color_attachments: Box<[vk::AttachmentReference]>,
    _depth_attachment: Option<Box<vk::AttachmentReference>>,
    _input_attachments: Box<[vk::AttachmentReference]>,
}

impl Subpass {
    pub fn new(desc: SubpassDesc) -> Subpass {
        let color_attachments = desc.color_attachments.into_boxed_slice();
        let depth_attachment = desc.depth_attachment.map(Box::new);
        let input_attachments = desc.input_attachments.into_boxed_slice();

        let mut raw = vk::SubpassDescription::default();
        raw.pipeline_bind_point = desc.bind_point.unwrap();
        raw.flags = desc.flags.unwrap_or(vk::SubpassDescriptionFlags::empty());

        if !color_attachments.is_empty() {
            raw.color_attachment_count = color_attachments.len() as u32;
            raw.p_color_attachments = color_attachments.as_ptr();
        }

        if let Some(ref depth) = depth_attachment {
            raw.p_depth_stencil_attachment = depth.as_ref() as *const _;
        }

        if !input_attachments.is_empty() {
            raw.input_attachment_count = input_attachments.len() as u32;
            raw.p_input_attachments = input_attachments.as_ptr();
        }

        Subpass {
            raw,
            _color_attachments: color_attachments,
            _depth_attachment: depth_attachment,
            _input_attachments: input_attachments,
        }
    }
}

pub struct SubpassDesc {
    color_attachments: Vec<vk::AttachmentReference>,
    depth_attachment: Option<vk::AttachmentReference>,
    input_attachments: Vec<vk::AttachmentReference>,
    bind_point: Option<vk::PipelineBindPoint>,
    flags: Option<vk::SubpassDescriptionFlags>,
}

impl SubpassDesc {
    pub fn empty() -> Self {
        Self {
            color_attachments: Vec::new(),
            depth_attachment: None,
            input_attachments: Vec::new(),
            bind_point: None,
            flags: None,
        }
    }

    pub fn add_color_attachment_ref(mut self, attachment: vk::AttachmentReference) -> Self {
        self.color_attachments.push(attachment);
        self
    }

    pub fn color_attachments(mut self, attachments: Vec<vk::AttachmentReference>) -> Self {
        self.color_attachments = attachments;
        self
    }

    pub fn with_bind_point(mut self, bind_point: vk::PipelineBindPoint) -> Self {
        self.bind_point = Some(bind_point);
        self
    }

    pub fn add_depth_attachment_ref(mut self, attachment: vk::AttachmentReference) -> Self {
        self.depth_attachment = Some(attachment);
        self
    }

    pub fn add_input_attachment_ref(mut self, attachment: vk::AttachmentReference) -> Self {
        self.input_attachments.push(attachment);
        self
    }

    pub fn input_attachments(mut self, attachments: Vec<vk::AttachmentReference>) -> Self {
        self.input_attachments = attachments;
        self
    }

    pub fn flags(mut self, flags: vk::SubpassDescriptionFlags) -> Self {
        self.flags = Some(flags);
        self
    }
}
