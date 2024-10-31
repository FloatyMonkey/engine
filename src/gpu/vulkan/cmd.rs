use super::*;
use crate::gpu;

impl CmdList {
	fn cmd(&self) -> vk::CommandBuffer {
		self.command_buffers[self.bb_index]
	}
}

impl gpu::CmdListImpl<Device> for CmdList {
	fn reset(&mut self, device: &Device, surface: &Surface) {
		todo!()
	}

	fn copy_buffer(
		&self,
		src: &Buffer,
		src_offset: u64,
		dst: &Buffer,
		dst_offset: u64,
		size: u64,
	) {
		unsafe {
			self.device.cmd_copy_buffer(self.cmd(), src.buffer, dst.buffer, &[vk::BufferCopy {
				src_offset,
				dst_offset,
				size,
			}]);
		}
	}

	fn copy_texture(
		&self,
		src: &Texture,
		src_mip_level: u32,
		src_array_slice: u32,
		src_offset: [u32; 3],
		dst: &Texture,
		dst_mip_level: u32,
		dst_array_slice: u32,
		dst_offset: [u32; 3],
		size: [u32; 3],
	) {
		unsafe {
			self.device.cmd_copy_image(self.cmd(), src.image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, dst.image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[vk::ImageCopy {
				src_subresource: vk::ImageSubresourceLayers {
					aspect_mask: vk::ImageAspectFlags::COLOR, // TODO: Correct aspects
					mip_level: src_mip_level,
					base_array_layer: src_array_slice,
					layer_count: 1,
				},
				src_offset: map_offset(&src_offset),
				dst_subresource: vk::ImageSubresourceLayers {
					aspect_mask: vk::ImageAspectFlags::COLOR, // TODO: Correct aspects
					mip_level: dst_mip_level,
					base_array_layer: dst_array_slice,
					layer_count: 1,
				},
				dst_offset: map_offset(&dst_offset),
				extent: map_extent(&size),
			}]);
		}
	}

	fn copy_buffer_to_texture(
		&self,
		src: &Buffer,
		src_offset: u64,
		src_bytes_per_row: u32,
		dst: &Texture,
		dst_mip_level: u32,
		dst_array_slice: u32,
		dst_offset: [u32; 3],
		size: [u32; 3],
	) {
		unsafe {
			self.device.cmd_copy_buffer_to_image(self.cmd(), src.buffer, dst.image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[vk::BufferImageCopy {
				buffer_offset: src_offset,
				buffer_row_length: src_bytes_per_row,
				buffer_image_height: 0,
				image_subresource: vk::ImageSubresourceLayers {
					aspect_mask: vk::ImageAspectFlags::COLOR, // TODO: Correct aspects
					mip_level: dst_mip_level,
					base_array_layer: dst_array_slice,
					layer_count: 1,
				},
				image_offset: map_offset(&dst_offset),
				image_extent: map_extent(&size),
			}]);
		}
	}

	fn copy_texture_to_buffer(
		&self,
		src: &Texture,
		src_mip_level: u32,
		src_array_slice: u32,
		src_offset: [u32; 3],
		dst: &Buffer,
		dst_offset: u64,
		dst_bytes_per_row: u32,
		size: [u32; 3],
	) {
		unsafe {
			self.device.cmd_copy_image_to_buffer(self.cmd(), src.image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, dst.buffer, &[vk::BufferImageCopy {
				buffer_offset: dst_offset,
				buffer_row_length: dst_bytes_per_row,
				buffer_image_height: 0,
				image_subresource: vk::ImageSubresourceLayers {
					aspect_mask: vk::ImageAspectFlags::COLOR, // TODO: Correct aspects
					mip_level: src_mip_level,
					base_array_layer: src_array_slice,
					layer_count: 1,
				},
				image_offset: map_offset(&src_offset),
				image_extent: map_extent(&size),
			}]);
		}
	}

	fn render_pass_begin(&self, desc: &gpu::RenderPassDesc<Device>) {
		let color_attachments = desc.colors.iter().map(|target| {
			let (load_op, clear) = map_load_op(target.load_op);
			vk::RenderingAttachmentInfo::default()
				.image_view(target.texture.rtv.unwrap())
				.image_layout(vk::ImageLayout::GENERAL) // TODO: COLOR_ATTACHMENT_OPTIMAL or others
				.load_op(load_op)
				.store_op(map_store_op(target.store_op))
				.clear_value(vk::ClearValue {
					color: vk::ClearColorValue {
						float32: clear.into(),
					},
				})
		}).collect::<Vec<_>>();

		let depth_stencil = desc.depth_stencil.as_ref().map(|target| {
			let (load_op, clear) = map_load_op(target.load_op);
			vk::RenderingAttachmentInfo::default()
				.image_view(target.texture.dsv.unwrap())
				.image_layout(vk::ImageLayout::GENERAL) // TODO: DEPTH_STENCIL_ATTACHMENT_OPTIMAL or others
				.load_op(load_op)
				.store_op(map_store_op(target.store_op))
				.clear_value(vk::ClearValue {
					depth_stencil: vk::ClearDepthStencilValue {
						depth: clear.0,
						stencil: clear.1 as u32,
					},
				})
		});

		let mut vk_info = vk::RenderingInfo::default()
			.layer_count(1)
			.color_attachments(&color_attachments);

		if let Some(ref depth_stencil) = depth_stencil {
			vk_info = vk_info.depth_attachment(depth_stencil);
			vk_info = vk_info.stencil_attachment(depth_stencil); // TODO: Conditionally enable
		}

		unsafe {
			self.device.cmd_begin_rendering(self.cmd(), &vk_info);
		}
	}

	fn render_pass_end(&self) {
		unsafe {
			self.device.cmd_end_rendering(self.cmd());
		}
	}

	fn barriers(&self, barriers: &gpu::Barriers<Device>) {
		let memory_barriers = barriers.global.iter().map(|barrier| {
			vk::MemoryBarrier2::default()
		}).collect::<Vec<_>>();

		let buffer_memory_barriers = barriers.buffer.iter().map(|barrier| {
			vk::BufferMemoryBarrier2::default()
				.buffer(barrier.buffer.buffer)
		}).collect::<Vec<_>>();

		let image_memory_barriers = barriers.texture.iter().map(|barrier| {
			vk::ImageMemoryBarrier2::default()
				.old_layout(map_image_layout(barrier.old_layout))
				.new_layout(map_image_layout(barrier.new_layout))
				.image(barrier.texture.image)
		}).collect::<Vec<_>>();

		let dependency_info = vk::DependencyInfo::default()
			.memory_barriers(&memory_barriers)
			.buffer_memory_barriers(&buffer_memory_barriers)
			.image_memory_barriers(&image_memory_barriers);

		unsafe {
			self.device.cmd_pipeline_barrier2(self.cmd(), &dependency_info);
		}

		panic!("Not fully implemented");
	}

	fn set_viewport(&self, rect: &gpu::Rect<f32>, depth: Range<f32>) {
		let vk_viewport = vk::Viewport {
			x: rect.left,
			y: rect.bottom,
			width: rect.right - rect.left,
			height: rect.top - rect.bottom,
			min_depth: depth.start,
			max_depth: depth.end,
		};

		unsafe {
			self.device.cmd_set_viewport(self.cmd(), 0, &[vk_viewport]);
		}
	}

	fn set_scissor(&self, rect: &gpu::Rect<u32>) {
		let vk_rect = vk::Rect2D {
			offset: vk::Offset2D {
				x: rect.left as _,
				y: rect.top as _,
			},
			extent: vk::Extent2D {
				width: (rect.right - rect.left),
				height: (rect.top - rect.bottom),
			},
		};

		unsafe {
			self.device.cmd_set_scissor(self.cmd(), 0, &[vk_rect]);
		}
	}

	fn set_blend_constant(&self, color: gpu::Color<f32>) {
		unsafe {
			self.device.cmd_set_blend_constants(self.cmd(), &[color.r, color.g, color.b, color.a]);
		}
	}

	fn set_stencil_reference(&self, reference: u32) {
		unsafe {
			self.device.cmd_set_stencil_reference(self.cmd(), vk::StencilFaceFlags::FRONT_AND_BACK, reference);
		}
	}

	fn set_index_buffer(&self, buffer: &Buffer, offset: u64, format: gpu::Format) {
		let vk_format = map_index_format(format);

		unsafe {
			self.device.cmd_bind_index_buffer(self.cmd(), buffer.buffer, offset, vk_format);
		}
	}

	fn set_graphics_pipeline(&self, pipeline: &GraphicsPipeline) {
		unsafe {
			self.device.cmd_bind_pipeline(self.cmd(), vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline);
		}
	}

	fn set_compute_pipeline(&self, pipeline: &ComputePipeline) {
		unsafe {
			self.device.cmd_bind_pipeline(self.cmd(), vk::PipelineBindPoint::COMPUTE, pipeline.pipeline);
		}
	}

	fn set_raytracing_pipeline(&self, pipeline: &RaytracingPipeline) {
		unsafe {
			self.device.cmd_bind_pipeline(self.cmd(), vk::PipelineBindPoint::RAY_TRACING_KHR, pipeline.pipeline);
		}
	}

	fn graphics_push_constants(&self, offset: u32, data: &[u8]) {
		/*unsafe {
			self.device.cmd_push_constants(
				self.cmd(),
				self.pipeline_layout,
				vk::ShaderStageFlags::ALL_GRAPHICS,
				0,
				data,
			);
		}*/
		todo!()
	}

	fn compute_push_constants(&self, offset: u32, data: &[u8]) {
		/*unsafe {
			self.device.cmd_push_constants(
				self.cmd(),
				self.pipeline_layout,
				vk::ShaderStageFlags::COMPUTE,
				0,
				data,
			);
		}*/
		todo!()
	}

	fn draw(&self, vertices: Range<u32>, instances: Range<u32>) {
		unsafe {
			self.device.cmd_draw(self.cmd(), vertices.len() as u32, instances.len() as u32, vertices.start, vertices.start);
		}
	}

	fn draw_indexed(&self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
		unsafe {
			self.device.cmd_draw_indexed(self.cmd(), indices.len() as u32, instances.len() as u32, indices.start, base_vertex, instances.start);
		}
	}

	fn dispatch(&self, groups: [u32; 3]) {
		unsafe {
			self.device.cmd_dispatch(self.cmd(), groups[0], groups[1], groups[2]);
		}
	}

	fn dispatch_rays(&self, desc: &gpu::DispatchRaysDesc) {
		unsafe {
			self.ray_tracing_pipeline_ext.cmd_trace_rays(
				self.cmd(),
				&desc.raygen.as_ref().map_or(Default::default(), |t| vk::StridedDeviceAddressRegionKHR {
					device_address: t.ptr.0,
					stride: t.stride as _,
					size: t.size as _,
				}),
				&desc.miss.as_ref().map_or(Default::default(), |t| vk::StridedDeviceAddressRegionKHR {
					device_address: t.ptr.0,
					stride: t.stride as _,
					size: t.size as _,
				}),
				&desc.hit_group.as_ref().map_or(Default::default(), |t| vk::StridedDeviceAddressRegionKHR {
					device_address: t.ptr.0,
					stride: t.stride as _,
					size: t.size as _,
				}),
				&desc.callable.as_ref().map_or(Default::default(), |t| vk::StridedDeviceAddressRegionKHR {
					device_address: t.ptr.0,
					stride: t.stride as _,
					size: t.size as _,
				}),
				desc.size[0],
				desc.size[1],
				desc.size[2],
			);
		}
	}

	fn build_acceleration_structure(&self, desc: &gpu::AccelerationStructureBuildDesc<Device>) {
		let mut info = AccelerationStructureInfo::build(desc.inputs);

		info.build_info.dst_acceleration_structure = desc.dst.acceleration_structure;
		info.build_info.scratch_data.device_address = desc.scratch_data.0;

		if let Some(src) = desc.src {
			info.build_info.src_acceleration_structure = src.acceleration_structure;
		}

		unsafe {
			let infos = [info.build_info];
			let build_range_infos: &[&[_]] = &[&info.build_range_infos];

			self.acceleration_structure_ext.cmd_build_acceleration_structures(
				self.cmd(),
				&infos,
				build_range_infos,
			);
		}
	}

	fn debug_marker(&self, name: &str, color: gpu::Color<u8>) {
		if let Some(ext) = &self.debug_utils_ext {
			let name = CString::new(name).unwrap();
			let label = vk::DebugUtilsLabelEXT::default()
				.label_name(&name)
				.color(color.to_f32().into());

			unsafe { ext.cmd_insert_debug_utils_label(self.cmd(), &label) };
		}
	}

	fn debug_event_push(&self, name: &str, color: gpu::Color<u8>) {
		if let Some(ext) = &self.debug_utils_ext {
			let name = CString::new(name).unwrap();
			let label = vk::DebugUtilsLabelEXT::default()
				.label_name(&name)
				.color(color.to_f32().into());

			unsafe { ext.cmd_begin_debug_utils_label(self.cmd(), &label) };
		}
	}

	fn debug_event_pop(&self) {
		if let Some(ext) = &self.debug_utils_ext {
			unsafe { ext.cmd_end_debug_utils_label(self.cmd()) };
		}
	}
}
