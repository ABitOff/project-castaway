use project_beacon::vulkan;

#[test]
fn test() {
    assert_eq!(
        vulkan::Version::from_major(2_087_830_939),
        vulkan::Version {
            variant: 0,
            major: 2_087_830_939,
            minor: 0,
            patch: 0
        }
    );
    assert_eq!(
        vulkan::Version::from_major_minor(2_087_830_939, 3_835_216_933),
        vulkan::Version {
            variant: 0,
            major: 2_087_830_939,
            minor: 3_835_216_933,
            patch: 0
        }
    );
    assert_eq!(
        vulkan::Version::new(2_087_830_939, 3_835_216_933, 42_366_749, 651_707_963),
        vulkan::Version {
            variant: 2_087_830_939,
            major: 3_835_216_933,
            minor: 42_366_749,
            patch: 651_707_963,
        }
    );
}
