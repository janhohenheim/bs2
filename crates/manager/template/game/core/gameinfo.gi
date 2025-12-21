"GameInfo"
{
	game 		"Core"
	title 		"Core"
	type		singleplayer_only
	GameData	"base.fgd"
	nodegraph 1
	perfwizard 1
	tonemapping 1 // Show tonemapping ui in tools mode

	FileSystem
	{
		//
		// The code that loads this file automatically does a few things here:
		//
		// 1. For each "Game" search path, it adds a "GameBin" path, in <dir>\bin
		// 2. For each "Game" search path, it adds another "Game" path in front of it with _<langage> at the end.
		//    For example: c:\hl2\cstrike on a french machine would get a c:\hl2\cstrike_french path added to it.
		// 3. For the first "Game" search path, it adds a search path called "MOD".
		// 4. For the first "Game" search path, it adds a search path called "DEFAULT_WRITE_PATH".
		//

		//
		// Search paths are relative to the exe directory\..\
		//
		SearchPaths
		{
			Game				core
			AddonRoot			core_addons

			Mod					core
		}
	}

	MaterialSystem2
	{
		RenderModes
		{
			"game" "Default"
			"game" "Depth"
			"game" "ProjectionDepth"
			"game" "PrepassLight"
			"game" "FullDeferredLight"
			"game" "DeferredGather"
			"game" "Forward"

			"tools" "ToolsVis" // Visualization modes for all shaders (lighting only, normal maps only, etc.)
			"tools" "ToolsWireframe" // This should use the ToolsVis mode above instead of being its own mode
			"tools" "ToolsUtil" // Meant to be used to render tools sceneobjects that are mod-independent, like the origin grid
		}
	}


	MaterialEditor
	{
		"DefaultShader" "csgo_simple"
		"DefaultAutoPromptForShaderOnNewMaterialCreation" "1"
	}

	Engine2
	{
		"HasModAppSystems" "0"
		"Capable64Bit" "1"
		"DefaultToolsRenderSystem"				"-dx11"
	}

	InputSystem
	{
		"ButtonCodeIsScanCode"		"1"
		"LockButtonCodeIsScanCode"	"1"
	}

	ToolsEnvironment
	{
		"Engine"	"Source 2"
		"ToolsDir"	"../sdktools"	// NOTE: Default Tools path. This is relative to the mod path.
	}

	Hammer
	{
		"fgd"					"base.fgd"	// NOTE: This is relative to the 'mod' path.
		"GameFeatureSet"				"CounterStrike"
		"DefaultTextureScale"			"0.125000"
		"DefaultSolidEntity"			"trigger_multiple"
		"DefaultPointEntity"			"info_player_start"
		"NavMarkupEntity"				"func_nav_markup"
		"RenderMode"					"ToolsVis"
		"TileMeshesEnabled"				"1"
		"TileGridSupportsBlendHeight"	"1"
		"TileGridBlendDefaultColor"		"0 255 0"
		"LoadScriptEntities"			"0"
		"UsesBakedLighting"				"1"
		"ShadowAtlasWidth"				"6144"
		"ShadowAtlasHeight"				"6144"
		"TimeSlicedShadowMapRendering"	"1"
		"TerrainTools"					"1"
		"DefaultGrassMaterial"			"materials/grass/grassquad1.vmat"
		"SteamAudioEnabled"				"1"
		"AddonMapCommand"				"map_workshop"
		"LatticeDeformerEnabled"		"1"
		"SmartPropInstanceRendering"	"1"
	}

	SoundTool
	{
		"DefaultSoundEventType" "core_simple_3d"
	}

	RenderPipelineAliases
	{
		"Tools"			"Forward"
		"EnvMapBake"	"Forward"
	}


	ResourceCompiler
	{
		// Overrides of the default builders as specified in code, this controls which map builder steps
		// will be run when resource compiler is run for a map without specifiying any specific map builder
		// steps. Additionally this controls which builders are displayed in the hammer build dialog.
		DefaultMapBuilders
		{
			"bakedlighting"	"1"	// Enable lightmapping during compile time
			"nav"		"1"	// Generate nav mesh data
			"light"		"0"	// Using per-vertex indirect lighting baked from within hammer
			"envmap"	"0"	// this is broken
			"sareverb"	"0" // Bake Steam Audio reverb
			"sapaths"	"0" // Bake Steam Audio pathing info
			"sacustomdata"	"0"	// Bake Steam Audio custom data
		}

		CompileManifest
		{
			EnforceValidManifestResourcePaths "1"
		}

		MeshCompiler
		{
			PerDrawCullingData      "1"
			EncodeVertexBuffer      "1"
			EncodeIndexBuffer       "1"
			UseMikkTSpace           "1"
			MeshletConeWeight       ".15"
			SplitDepthStream		"1"
		}

		WorldRendererBuilder
		{
			FixTJunctionEdgeCracks  		"1"
			VisibilityGuidedMeshClustering		"1"
			MinimumTrianglesPerClusteredMesh	"2048"
			MinimumVerticesPerClusteredMesh		"2048"
			MinimumVolumePerClusteredMesh		"1800"		// ~12x12x12 cube
			MaxPrecomputedVisClusterMembership	"16"
			UseAggregateInstances			"1"
			AggregateInstancingMeshlets			"1"
			UseStaticEnvMapForObjectsWithLightingOrigin	"1"
		}

// Optimisation for Hammer Mesh Physics

		PhysicsBuilder
		{
			DefaultHammerMeshSimplification		"0.0"
		}

		BakedLighting
		{
			Version 2
			DeterministicBuild 1
			DisableCullingForShadows 1
			MinSpecLightmapSize 4096
            ImportanceVolumeTransitionRegion 120            // distance we transition from high to low resolution charts
                                                            // when a triangle is outside an importance volume
			LPVAtlas 1
			LPVOctree 0
			LightmapChannels
			{
				irradiance 1
				direct_light_shadows 1

				directional_irradiance
				{
					MaxResolution 4096
					CompressedFormat DXT1
				}

				debug_chart_color
				{
					MaxResolution 4096
					CompressedFormat DXT1
				}
			}
		}

		VisBuilder
		{
			MaxVisClusters "4096"
			PreMergeOpenSpaceDistanceThreshold "128.0"
			PreMergeOpenSpaceMaxDimension "2048.0"
			PreMergeOpenSpaceMaxRatio "8.0"
			PreMergeSmallRegionsSizeThreshold "20.0"
			DeterministicBuild "1"
		}

		SteamAudio
		{
			ReverbDefaults
			{
				GridGenerationType	"0"						// 0: Automatic, Everywhere, 1: Automatic, Use Probe Generation Volume, 2: Manual
				FilterUsingVolumes	"1"						// Filter Using Probe Exclusion Volumes ( boolean )
				FilterUsingNavMesh	"0"						// Filter Using NavMesh
				GridSpacing			"3.0"
				HeightAboveFloor	"1.5"
				RebakeOption		"0"						// 0: cleanup, 1: manual, 2: auto
				NumRays				"32768"
				NumBounces			"64"
				IRDuration			"1.0"
				AmbisonicsOrder		"1"
				ClusteringEnabled	"0"
				ClusteringCubemapResolution	"16.0"
				ClusteringDepthThreshold	"10.0"
			}
			PathingDefaults
			{
				GridGenerationType	"0"						// 0: Automatic, Everywhere, 1: Automatic, Use Probe Generation Volume, 2: Manual
				FilterUsingVolumes	"1"						// Filter Using Probe Exclusion Volumes ( boolean )
				FilterUsingNavMesh	"0"						// Filter Using NavMesh
				GridSpacing			"3.0"
				HeightAboveFloor	"1.5"
				RebakeOption		"0"						// 0: cleanup, 1: manual, 2: auto
				NumVisSamples		"1"
				ProbeVisRadius		"0"
				ProbeVisThreshold	"0.1"
				ProbeVisPathRange	"1000.0"
			}
			CustomDataDefaults
			{
				GridGenerationType	"0"						// 0: Automatic, Everywhere, 1: Automatic, Use Probe Generation Volume, 2: Manual
				FilterUsingVolumes	"1"						// Filter Using Probe Exclusion Volumes ( boolean )
				FilterUsingNavMesh	"0"						// Filter Using NavMesh
				GridSpacing			"3.0"
				HeightAboveFloor	"1.5"
				RebakeOption		"0"						// 0: cleanup, 1: manual, 2: auto
				BakeOcclusion		"0"						// 0: Disabled, 1: Enabled
				BakeDimensions		"0"						// 0: Disabled, 1: Enabled
				BakeMaterials		"0"						// 0: Disabled, 1: Enabled
			}
		}

		TextureCompiler
		{
			//Compressor              "lz4"
			Compressor              "mermaid"
			//Compressor              "kraken"
			CompressMipsOnDisk      "1"
			CompressMinRatio        "95"
			AllowNP2Textures		"1"
			AllowPanoramaMipGeneration	"1"
			PublicToolsDefaultMaxRes "2048"
		}
	}

	ModelDoc
	{
		"models_gamedata"	"models_base.fgd"
		"features"			"animgraph"
	}

	Manifest
	{
		"GenerateVPKManifest" "1"
	}

	Memory
	{
		"EstimatedMaxCPUMemUsageMB"	"3388"
		"EstimatedMinGPUMemUsageMB"	"1246"
	}
	ConVars
	{
		snd_envelope_rate 100
		snd_soundmixer_update_maximum_frame_rate 0
	}
}
