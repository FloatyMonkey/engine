use super::*;
use crate as gpu;

impl CmdList {
	fn cmd(&self) -> &ID3D12GraphicsCommandList7 {
		&self.command_lists[self.bb_index]
	}
}

impl gpu::CmdListImpl<Device> for CmdList {
	fn reset(&mut self, device: &Device, surface: &Surface) {
		let bb = unsafe { surface.swap_chain.GetCurrentBackBufferIndex() as usize };
		self.bb_index = bb;
		if surface.frame_fence_value[bb] != 0 {
			unsafe {
				self.command_allocators[bb].Reset().unwrap();
				self.command_lists[bb].Reset(&self.command_allocators[bb], None).unwrap();
			}
		}

		unsafe {
			self.cmd().SetDescriptorHeaps(&[
				Some(device.resource_heap.heap.clone()),
				Some(device.sampler_heap.heap.clone()),
			]);
		}
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
			self.cmd().CopyBufferRegion(&dst.resource, dst_offset, &src.resource, src_offset, size);
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
			self.cmd().CopyTextureRegion(
				&D3D12_TEXTURE_COPY_LOCATION {
					pResource: std::mem::transmute_copy(&dst.resource),
					Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
					Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
						// TODO: Account for plane slices (aspects): color = 0, depth = 0, stencil = 1
						// https://learn.microsoft.com/en-us/windows/win32/direct3d12/subresources#plane-slice
						// subresource_index = mip_level + (array_slice + plane * desc.array_size) * desc.mip_levels
						// Also update the other copy_texture functions
						SubresourceIndex: dst_mip_level + dst_array_slice * dst.resource.GetDesc().MipLevels as u32,
					},
				},
				dst_offset[0],
				dst_offset[1],
				dst_offset[2],
				&D3D12_TEXTURE_COPY_LOCATION {
					pResource: std::mem::transmute_copy(&src.resource),
					Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
					Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
						SubresourceIndex: src_mip_level + src_array_slice * src.resource.GetDesc().MipLevels as u32,
					},
				},
				Some(&map_box(&src_offset, &size)),
			);
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
			self.cmd().CopyTextureRegion(
				&D3D12_TEXTURE_COPY_LOCATION {
					pResource: std::mem::transmute_copy(&dst.resource),
					Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
					Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
						SubresourceIndex: dst_mip_level + dst_array_slice * dst.resource.GetDesc().MipLevels as u32,
					},
				},
				dst_offset[0],
				dst_offset[1],
				dst_offset[2],
				&D3D12_TEXTURE_COPY_LOCATION {
					pResource: std::mem::transmute_copy(&src.resource),
					Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
					Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
						PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
							Offset: src_offset,
							Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
								Format: dst.resource.GetDesc().Format,
								Width: size[0],
								Height: size[1],
								Depth: size[2],
								RowPitch: src_bytes_per_row,
							},
						},
					},
				},
				None,
			);
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
			self.cmd().CopyTextureRegion(
				&D3D12_TEXTURE_COPY_LOCATION {
					pResource: std::mem::transmute_copy(&dst.resource),
					Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
					Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
						PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
							Offset: dst_offset,
							Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
								Format: src.resource.GetDesc().Format,
								Width: size[0],
								Height: size[1],
								Depth: size[2],
								RowPitch: dst_bytes_per_row,
							},
						},
					},
				},
				0,
				0,
				0,
				&D3D12_TEXTURE_COPY_LOCATION {
					pResource: std::mem::transmute_copy(&src.resource),
					Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
					Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
						SubresourceIndex: src_mip_level + src_array_slice * src.resource.GetDesc().MipLevels as u32,
					},
				},
				Some(&map_box(&src_offset, &size)),
			);
		}
	}

	fn render_pass_begin(&self, desc: &gpu::RenderPassDesc<Device>) {
		let rt = desc.colors.iter().map(|target| {
			let (load_op, clear) = map_load_op(target.load_op);
			let resource_desc = unsafe { target.texture.resource.GetDesc() };

			let begin = D3D12_RENDER_PASS_BEGINNING_ACCESS {
				Type: load_op,
				Anonymous: D3D12_RENDER_PASS_BEGINNING_ACCESS_0 {
					Clear: D3D12_RENDER_PASS_BEGINNING_ACCESS_CLEAR_PARAMETERS {
						ClearValue: D3D12_CLEAR_VALUE {
							Format: resource_desc.Format,
							Anonymous: D3D12_CLEAR_VALUE_0 {
								Color: clear.into(),
							},
						},
					},
				},
			};

			let end = D3D12_RENDER_PASS_ENDING_ACCESS {
				Type: map_store_op(target.store_op),
				Anonymous: D3D12_RENDER_PASS_ENDING_ACCESS_0 {
					Resolve: Default::default(),
				},
			};

			D3D12_RENDER_PASS_RENDER_TARGET_DESC {
				cpuDescriptor: target.texture.rtv.unwrap(),
				BeginningAccess: begin,
				EndingAccess: end,
			}
		}).collect::<Vec<_>>();

		let ds = desc.depth_stencil.as_ref().map(|target| {
			let (load_op, clear) = map_load_op(target.load_op);
			let resource_desc = unsafe { target.texture.resource.GetDesc() };

			let depth_begin = D3D12_RENDER_PASS_BEGINNING_ACCESS {
				Type: load_op,
				Anonymous: D3D12_RENDER_PASS_BEGINNING_ACCESS_0 {
					Clear: D3D12_RENDER_PASS_BEGINNING_ACCESS_CLEAR_PARAMETERS {
						ClearValue: D3D12_CLEAR_VALUE {
							Format: resource_desc.Format,
							Anonymous: D3D12_CLEAR_VALUE_0 {
								DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
									Depth: clear.0,
									Stencil: clear.1,
								},
							},
						},
					},
				},
			};

			let depth_end = D3D12_RENDER_PASS_ENDING_ACCESS {
				Type: map_store_op(target.store_op),
				Anonymous: D3D12_RENDER_PASS_ENDING_ACCESS_0 {
					Resolve: Default::default(),
				},
			};

			let stencil_begin = D3D12_RENDER_PASS_BEGINNING_ACCESS {
				Type: load_op,
				Anonymous: D3D12_RENDER_PASS_BEGINNING_ACCESS_0 {
					Clear: D3D12_RENDER_PASS_BEGINNING_ACCESS_CLEAR_PARAMETERS {
						ClearValue: D3D12_CLEAR_VALUE {
							Format: resource_desc.Format,
							Anonymous: D3D12_CLEAR_VALUE_0 {
								DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
									Depth: clear.0,
									Stencil: clear.1,
								},
							},
						},
					},
				},
			};

			let stencil_end = D3D12_RENDER_PASS_ENDING_ACCESS {
				Type: map_store_op(target.store_op),
				Anonymous: D3D12_RENDER_PASS_ENDING_ACCESS_0 {
					Resolve: Default::default(),
				},
			};

			D3D12_RENDER_PASS_DEPTH_STENCIL_DESC {
				cpuDescriptor: target.texture.dsv.unwrap(),
				DepthBeginningAccess: depth_begin,
				StencilBeginningAccess: stencil_begin,
				DepthEndingAccess: depth_end,
				StencilEndingAccess: stencil_end,
			}
		});

		unsafe {
			self.cmd().BeginRenderPass(Some(rt.as_slice()), ds.map(|ds| &ds as *const _), D3D12_RENDER_PASS_FLAG_NONE);
		}
	}

	fn render_pass_end(&self) {
		unsafe {
			self.cmd().EndRenderPass();
		}
	}

	fn barriers(&self, barriers: &gpu::Barriers<Device>) {
		let global_barriers = barriers.global.iter().map(|_| D3D12_GLOBAL_BARRIER {
			SyncBefore: D3D12_BARRIER_SYNC_ALL,
			SyncAfter: D3D12_BARRIER_SYNC_ALL,
			AccessBefore: D3D12_BARRIER_ACCESS_UNORDERED_ACCESS,
			AccessAfter: D3D12_BARRIER_ACCESS_UNORDERED_ACCESS,
		}).collect::<Vec<_>>();

		let buffer_barriers = barriers.buffer.iter().map(|barrier| D3D12_BUFFER_BARRIER {
			SyncBefore: D3D12_BARRIER_SYNC_ALL,
			SyncAfter: D3D12_BARRIER_SYNC_ALL,
			AccessBefore: D3D12_BARRIER_ACCESS_COMMON,
			AccessAfter: D3D12_BARRIER_ACCESS_COMMON,
			pResource: unsafe { std::mem::transmute_copy(&barrier.buffer.resource) },
			Offset: 0,
			Size: u64::MAX,
		}).collect::<Vec<_>>();

		let texture_barriers = barriers.texture.iter().map(|barrier| D3D12_TEXTURE_BARRIER {
			SyncBefore: D3D12_BARRIER_SYNC_ALL,
			SyncAfter: D3D12_BARRIER_SYNC_ALL,
			AccessBefore: D3D12_BARRIER_ACCESS_COMMON,
			AccessAfter: D3D12_BARRIER_ACCESS_COMMON,
			LayoutBefore: map_texture_layout(barrier.old_layout),
			LayoutAfter: map_texture_layout(barrier.new_layout),
			pResource: unsafe { std::mem::transmute_copy(&barrier.texture.resource) },
			Subresources: D3D12_BARRIER_SUBRESOURCE_RANGE {
				IndexOrFirstMipLevel: 0xffffffff, // All subresources
				NumMipLevels: 0,
				FirstArraySlice: 0,
				NumArraySlices: 0,
				FirstPlane: 0,
				NumPlanes: 0,
			},
			Flags: D3D12_TEXTURE_BARRIER_FLAG_NONE,
		}).collect::<Vec<_>>();

		let barrier_groups = [
			D3D12_BARRIER_GROUP {
				Type: D3D12_BARRIER_TYPE_GLOBAL,
				NumBarriers: global_barriers.len() as u32,
				Anonymous: D3D12_BARRIER_GROUP_0 {
					pGlobalBarriers: global_barriers.as_ptr(),
				},
			},
			D3D12_BARRIER_GROUP {
				Type: D3D12_BARRIER_TYPE_BUFFER,
				NumBarriers: buffer_barriers.len() as u32,
				Anonymous: D3D12_BARRIER_GROUP_0 {
					pBufferBarriers: buffer_barriers.as_ptr(),
				},
			},
			D3D12_BARRIER_GROUP {
				Type: D3D12_BARRIER_TYPE_TEXTURE,
				NumBarriers: texture_barriers.len() as u32,
				Anonymous: D3D12_BARRIER_GROUP_0 {
					pTextureBarriers: texture_barriers.as_ptr(),
				},
			},
		];

		unsafe {
			self.cmd().Barrier(&barrier_groups);
		}
	}

	fn set_viewport(&self, rect: &gpu::Rect<f32>, depth: Range<f32>) {
		let dx_viewport = D3D12_VIEWPORT {
			TopLeftX: rect.left,
			TopLeftY: rect.top,
			Width: rect.right - rect.left,
			Height: rect.bottom - rect.top,
			MinDepth: depth.start,
			MaxDepth: depth.end,
		};

		unsafe {
			self.cmd().RSSetViewports(&[dx_viewport]);
		}
	}

	fn set_scissor(&self, rect: &gpu::Rect<u32>) {
		let dx_rect = RECT {
			left: rect.left as i32,
			top: rect.top as i32,
			right: rect.right as i32,
			bottom: rect.bottom as i32,
		};
		
		unsafe {
			self.cmd().RSSetScissorRects(&[dx_rect]);
		}
	}

	fn set_blend_constant(&self, color: gpu::Color<f32>) {
		unsafe {
			self.cmd().OMSetBlendFactor(Some(&color.into()));
		}
	}

	fn set_stencil_reference(&self, reference: u32) {
		unsafe {
			self.cmd().OMSetStencilRef(reference);
		}
	}

	fn set_index_buffer(&self, buffer: &Buffer, offset: u64, format: gpu::Format) {
		let ibv = D3D12_INDEX_BUFFER_VIEW {
			BufferLocation: unsafe { buffer.resource.GetGPUVirtualAddress() },
			SizeInBytes: (buffer.size - offset) as u32,
			Format: map_format(format),
		};

		unsafe {
			self.cmd().IASetIndexBuffer(Some(&ibv));
		}
	}

	fn set_graphics_pipeline(&self, pipeline: &GraphicsPipeline) {
		let cmd = self.cmd();
		unsafe {
			cmd.SetGraphicsRootSignature(&pipeline.root_signature);
			cmd.SetPipelineState(&pipeline.pipeline_state);
			cmd.IASetPrimitiveTopology(pipeline.topology);
			cmd.SetGraphicsRootDescriptorTable(1, self.resource_heap_base);
		}
	}

	fn set_compute_pipeline(&self, pipeline: &ComputePipeline) {
		let cmd = self.cmd();
		unsafe {
			cmd.SetComputeRootSignature(&pipeline.root_signature);
			cmd.SetPipelineState(&pipeline.pipeline_state);
			cmd.SetComputeRootDescriptorTable(1, self.resource_heap_base);
		}
	}

	fn set_raytracing_pipeline(&self, pipeline: &RaytracingPipeline) {
		let cmd = self.cmd();
		unsafe {
			cmd.SetComputeRootSignature(&pipeline.root_signature);
			cmd.SetPipelineState1(&pipeline.state_object);
			cmd.SetComputeRootDescriptorTable(1, self.resource_heap_base);
		}
	}

	fn graphics_push_constants(&self, offset: u32, data: &[u8]) {
		assert_eq!(offset % 4, 0);
		assert_eq!(data.len() % 4, 0);
		
		unsafe {
			self.cmd().SetGraphicsRoot32BitConstants(0, data.len() as u32 / 4, data.as_ptr() as *const _, offset / 4);
		}
	}

	fn compute_push_constants(&self, offset: u32, data: &[u8]) {
		assert_eq!(offset % 4, 0);
		assert_eq!(data.len() % 4, 0);

		unsafe {
			self.cmd().SetComputeRoot32BitConstants(0, data.len() as u32 / 4, data.as_ptr() as *const _, offset / 4);
		}
	}

	fn draw(&self, vertices: Range<u32>, instances: Range<u32>) {
		unsafe {
			self.cmd().DrawInstanced(vertices.len() as u32, instances.len() as u32, vertices.start, instances.start);
		}
	}

	fn draw_indexed(&self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
		unsafe {
			self.cmd().DrawIndexedInstanced(indices.len() as u32, instances.len() as u32, indices.start, base_vertex, instances.start);
		}
	}

	fn dispatch(&self, groups: [u32; 3]) {
		unsafe {
			self.cmd().Dispatch(groups[0], groups[1], groups[2]);
		}
	}

	fn dispatch_rays(&self, desc: &gpu::DispatchRaysDesc) {
		let dx_desc = D3D12_DISPATCH_RAYS_DESC {
			RayGenerationShaderRecord: desc.raygen.as_ref().map_or(Default::default(), |t| D3D12_GPU_VIRTUAL_ADDRESS_RANGE {
				StartAddress: t.ptr.0,
				SizeInBytes: t.size as _,
			}),
			MissShaderTable: desc.miss.as_ref().map_or(Default::default(), |t| D3D12_GPU_VIRTUAL_ADDRESS_RANGE_AND_STRIDE {
				StartAddress: t.ptr.0,
				SizeInBytes: t.size as _,
				StrideInBytes: t.stride as _,
			}),
			HitGroupTable: desc.hit_group.as_ref().map_or(Default::default(), |t| D3D12_GPU_VIRTUAL_ADDRESS_RANGE_AND_STRIDE {
				StartAddress: t.ptr.0,
				SizeInBytes: t.size as _,
				StrideInBytes: t.stride as _,
			}),
			CallableShaderTable: desc.callable.as_ref().map_or(Default::default(), |t| D3D12_GPU_VIRTUAL_ADDRESS_RANGE_AND_STRIDE {
				StartAddress: t.ptr.0,
				SizeInBytes: t.size as _,
				StrideInBytes: t.stride as _,
			}),
			Width: desc.size[0],
			Height: desc.size[1],
			Depth: desc.size[2],
		};

		unsafe {
			self.cmd().DispatchRays(&dx_desc);
		}
	}

	fn build_acceleration_structure(&self, desc: &gpu::AccelerationStructureBuildDesc<Device>) {
		let info = AccelerationStructureInfo::build(desc.inputs);

		// TODO: D3D12_RAYTRACING_ACCELERATION_STRUCTURE_BUILD_FLAG_PERFORM_UPDATE if desc.src.is_some()

		let dx_desc = D3D12_BUILD_RAYTRACING_ACCELERATION_STRUCTURE_DESC {
			DestAccelerationStructureData: unsafe { desc.dst.resource.GetGPUVirtualAddress() },
			Inputs: info.desc,
			SourceAccelerationStructureData: desc.src.map_or(0, |b| unsafe { b.resource.GetGPUVirtualAddress() }),
			ScratchAccelerationStructureData: desc.scratch_data.0,
		};

		unsafe {
			self.cmd().BuildRaytracingAccelerationStructure(&dx_desc, None)
		}
	}

	fn debug_marker(&self, name: &str, color: gpu::Color<u8>) {
		if let Some(pix) = &self.pix {
			let color = 0xff000000 | (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32;
			pix.set_marker_on_command_list(self.cmd(), color as u64, name);
		}
	}

	fn debug_event_push(&self, name: &str, color: gpu::Color<u8>) {
		if let Some(pix) = &self.pix {
			let color = 0xff000000 | (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32;
			pix.begin_event_on_command_list(self.cmd(), color as u64, name);
		}
	}

	fn debug_event_pop(&self) {
		if let Some(pix) = &self.pix {
			pix.end_event_on_command_list(self.cmd());
		}
	}
}
