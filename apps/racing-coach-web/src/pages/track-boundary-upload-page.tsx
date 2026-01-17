import { useUploadTrackBoundary } from '@/api/generated/tracks/tracks';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { zodResolver } from '@hookform/resolvers/zod';
import { useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router';
import { z } from 'zod';

const uploadSchema = z.object({
  leftLapNumber: z.coerce.number().int().min(1, 'Must be at least 1'),
  rightLapNumber: z.coerce.number().int().min(1, 'Must be at least 1'),
  gridSize: z.coerce.number().int().min(100, 'Must be at least 100'),
});

export function TrackBoundaryUploadPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [file, setFile] = useState<File | null>(null);
  const [fileError, setFileError] = useState<string | null>(null);

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<z.input<typeof uploadSchema>, unknown, z.output<typeof uploadSchema>>({
    resolver: zodResolver(uploadSchema),
    defaultValues: {
      leftLapNumber: 1,
      rightLapNumber: 3,
      gridSize: 1000,
    },
  });

  const uploadMutation = useUploadTrackBoundary({
    mutation: {
      onSuccess: (data) => {
        queryClient.invalidateQueries({ queryKey: ['/api/v1/tracks'] });
        navigate(`/tracks/${data.boundary_id}`);
      },
    },
  });

  const onSubmit = async (data: z.output<typeof uploadSchema>) => {
    if (!file) {
      setFileError('Please select an IBT file');
      return;
    }

    setFileError(null);
    await uploadMutation.mutateAsync({
      data: {
        file,
        left_lap_number: data.leftLapNumber,
        right_lap_number: data.rightLapNumber,
        grid_size: data.gridSize,
      },
    });
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0];
    setFileError(null);

    if (!selectedFile) {
      setFile(null);
      return;
    }

    if (!selectedFile.name.toLowerCase().endsWith('.ibt')) {
      setFile(null);
      setFileError('Please select an IBT file (.ibt extension)');
      return;
    }

    setFile(selectedFile);
  };

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <div>
        <button
          onClick={() => navigate('/tracks')}
          className="flex items-center gap-1 mb-2 text-sm text-muted-foreground hover:text-foreground"
        >
          &larr; Back to Track Boundaries
        </button>
        <h2 className="text-3xl font-bold tracking-tight text-foreground">Upload Track Boundary</h2>
        <p className="text-muted-foreground">
          Upload an IBT file containing boundary laps to generate track boundary data
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>IBT File Upload</CardTitle>
          <CardDescription>
            The IBT file should contain at least two laps: one hugging the left side and one hugging
            the right side of the track. Follow the Garage61 collection method: drive an outlap, set
            a reset point, then drive left-hugging lap 1 and right-hugging lap 3.
          </CardDescription>
        </CardHeader>
        <form onSubmit={handleSubmit(onSubmit)}>
          <CardContent className="space-y-6">
            {uploadMutation.isError && (
              <div className="p-3 text-sm text-red-400 border border-red-800 rounded-md bg-red-900/20">
                {uploadMutation.error?.message ?? 'Failed to upload file'}
              </div>
            )}

            <div className="space-y-2">
              <Label htmlFor="file">IBT File</Label>
              <Input
                id="file"
                type="file"
                accept=".ibt"
                onChange={handleFileChange}
                className="cursor-pointer file:mr-4 file:py-1 file:px-3 file:rounded-md file:border-0 file:text-sm file:font-medium file:bg-muted file:text-foreground hover:file:bg-muted/80"
              />
              {fileError && <p className="text-sm text-red-400">{fileError}</p>}
              {file && (
                <p className="text-sm text-green-400">
                  Selected: {file.name} ({(file.size / 1024 / 1024).toFixed(2)} MB)
                </p>
              )}
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="leftLapNumber">Left Boundary Lap Number</Label>
                <Input id="leftLapNumber" type="number" min={1} {...register('leftLapNumber')} />
                {errors.leftLapNumber && (
                  <p className="text-sm text-red-400">{errors.leftLapNumber.message}</p>
                )}
                <p className="text-xs text-muted-foreground">
                  The lap where you drove along the left edge
                </p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="rightLapNumber">Right Boundary Lap Number</Label>
                <Input id="rightLapNumber" type="number" min={1} {...register('rightLapNumber')} />
                {errors.rightLapNumber && (
                  <p className="text-sm text-red-400">{errors.rightLapNumber.message}</p>
                )}
                <p className="text-xs text-muted-foreground">
                  The lap where you drove along the right edge
                </p>
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="gridSize">Grid Size</Label>
              <Input id="gridSize" type="number" min={100} {...register('gridSize')} />
              {errors.gridSize && (
                <p className="text-sm text-red-400">{errors.gridSize.message}</p>
              )}
              <p className="text-xs text-muted-foreground">
                Resolution of the boundary grid (default: 1000)
              </p>
            </div>
          </CardContent>
          <CardFooter>
            <Button type="submit" className="w-full" disabled={!file || uploadMutation.isPending}>
              {uploadMutation.isPending ? 'Uploading...' : 'Upload and Generate Boundary'}
            </Button>
          </CardFooter>
        </form>
      </Card>
    </div>
  );
}
