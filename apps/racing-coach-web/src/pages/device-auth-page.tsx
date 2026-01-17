import {
  useConfirmDeviceAuthorization,
  useGetDeviceAuthorizationStatus,
} from '@/api/generated/auth/auth';
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
import { Spinner } from '@/components/ui/loading-states';
import { zodResolver } from '@hookform/resolvers/zod';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useSearchParams } from 'react-router';
import { z } from 'zod';

const codeSchema = z.object({
  code: z
    .string()
    .length(8, 'Code must be exactly 8 characters')
    .regex(/^[A-Za-z0-9]+$/, 'Code must be alphanumeric'),
});

type CodeFormData = z.infer<typeof codeSchema>;

export function DeviceAuthPage() {
  const [searchParams] = useSearchParams();
  const initialCode = searchParams.get('code') || '';

  const [submittedCode, setSubmittedCode] = useState<string | null>(
    initialCode.length === 8 ? initialCode.toUpperCase() : null
  );
  const [actionResult, setActionResult] = useState<{
    type: 'success' | 'error';
    message: string;
  } | null>(null);

  const confirmMutation = useConfirmDeviceAuthorization();

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<CodeFormData>({
    resolver: zodResolver(codeSchema),
    defaultValues: { code: initialCode },
  });

  const deviceQuery = useGetDeviceAuthorizationStatus(submittedCode || '', {
    query: {
      enabled: !!submittedCode,
      retry: false,
    },
  });

  const onSubmitCode = (data: CodeFormData) => {
    setSubmittedCode(data.code.toUpperCase());
    setActionResult(null);
  };

  const handleAction = async (approve: boolean) => {
    if (!submittedCode) return;

    try {
      await confirmMutation.mutateAsync({
        data: { user_code: submittedCode, approve },
      });
      setActionResult({
        type: 'success',
        message: approve
          ? 'Device approved! You can close this page.'
          : 'Device denied.',
      });
    } catch {
      setActionResult({
        type: 'error',
        message: 'Failed to process request. Please try again.',
      });
    }
  };

  const resetForm = () => {
    setSubmittedCode(null);
    setActionResult(null);
  };

  // Show success/error result
  if (actionResult) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>
            {actionResult.type === 'success' ? 'Done' : 'Error'}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div
            className={`p-3 rounded-md border text-sm ${
              actionResult.type === 'success'
                ? 'bg-green-900/20 border-green-800 text-green-400'
                : 'bg-red-900/20 border-red-800 text-red-400'
            }`}
          >
            {actionResult.message}
          </div>
        </CardContent>
        <CardFooter>
          <Button variant="outline" onClick={resetForm} className="w-full">
            Authorize another device
          </Button>
        </CardFooter>
      </Card>
    );
  }

  // Show device info and approve/deny buttons
  if (submittedCode && deviceQuery.data) {
    const device = deviceQuery.data;
    const expiresAt = new Date(device.expires_at);
    const isExpired = expiresAt < new Date() || device.status === 'expired';
    const isAlreadyProcessed =
      device.status === 'authorized' || device.status === 'denied';

    return (
      <Card>
        <CardHeader>
          <CardTitle>Authorize Device</CardTitle>
          <CardDescription>
            A device is requesting access to your account
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {isExpired && (
            <div className="p-3 rounded-md bg-yellow-900/20 border border-yellow-800 text-yellow-400 text-sm">
              This authorization request has expired.
            </div>
          )}

          {isAlreadyProcessed && (
            <div className="p-3 rounded-md bg-teal-900/20 border border-teal-800 text-teal-400 text-sm">
              This device has already been {device.status}.
            </div>
          )}

          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-gray-400">Device</span>
              <span className="font-medium">{device.device_name}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-400">Code</span>
              <span className="font-mono">{submittedCode}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-400">Status</span>
              <span
                className={
                  device.status === 'pending'
                    ? 'text-yellow-400'
                    : device.status === 'authorized'
                      ? 'text-green-400'
                      : 'text-red-400'
                }
              >
                {device.status}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-400">Expires</span>
              <span>{expiresAt.toLocaleString()}</span>
            </div>
          </div>
        </CardContent>
        <CardFooter className="flex gap-3">
          {!isExpired && !isAlreadyProcessed ? (
            <>
              <Button
                variant="outline"
                className="flex-1"
                onClick={() => handleAction(false)}
                disabled={confirmMutation.isPending}
              >
                Deny
              </Button>
              <Button
                className="flex-1"
                onClick={() => handleAction(true)}
                disabled={confirmMutation.isPending}
              >
                {confirmMutation.isPending ? 'Processing...' : 'Approve'}
              </Button>
            </>
          ) : (
            <Button variant="outline" onClick={resetForm} className="w-full">
              Enter different code
            </Button>
          )}
        </CardFooter>
      </Card>
    );
  }

  // Show loading state while fetching device
  if (submittedCode && deviceQuery.isLoading) {
    return (
      <Card>
        <CardContent className="py-8 flex justify-center">
          <Spinner />
        </CardContent>
      </Card>
    );
  }

  // Show error if device not found
  if (submittedCode && deviceQuery.isError) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Device Not Found</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="p-3 rounded-md bg-red-900/20 border border-red-800 text-red-400 text-sm">
            No pending authorization found for code "{submittedCode}". Please
            check the code and try again.
          </div>
        </CardContent>
        <CardFooter>
          <Button variant="outline" onClick={resetForm} className="w-full">
            Try again
          </Button>
        </CardFooter>
      </Card>
    );
  }

  // Show code entry form
  return (
    <Card>
      <CardHeader>
        <CardTitle>Authorize Device</CardTitle>
        <CardDescription>
          Enter the code displayed on your desktop application
        </CardDescription>
      </CardHeader>
      <form onSubmit={handleSubmit(onSubmitCode)}>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="code">Device Code</Label>
            <Input
              id="code"
              type="text"
              placeholder="ABCD1234"
              maxLength={8}
              className="font-mono text-center text-lg tracking-widest uppercase"
              {...register('code')}
            />
            {errors.code && (
              <p className="text-sm text-red-400">{errors.code.message}</p>
            )}
          </div>
        </CardContent>
        <CardFooter>
          <Button type="submit" className="w-full">
            Check Device
          </Button>
        </CardFooter>
      </form>
    </Card>
  );
}
